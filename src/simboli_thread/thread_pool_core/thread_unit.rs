use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    thread::JoinHandle,
};

use crate::{ListCore, WaitingTask};

pub struct ThreadUnit<F, const N: usize>
where
    F: Fn() + 'static + Send,
{
    // thread
    // // unique
    pub(crate) id: usize,
    pub(crate) xorshift_seed: AtomicU32,
    // // engine
    pub(crate) spawn: Option<JoinHandle<()>>,
    pub(crate) running: AtomicPtr<WaitingTask<F>>,
    // // storage
    pub(crate) queue: AtomicPtr<[AtomicPtr<WaitingTask<F>>; N]>,
    pub(crate) batch: u32,
    pub(crate) top: AtomicUsize,
    pub(crate) bottom: AtomicUsize,
    // // flag
    pub(crate) threads_active: AtomicU64,
    pub(crate) empty_flag: AtomicBool,

    // share
    // // thread_pool
    pub(crate) total_threads: usize,
    pub(crate) pool: Arc<AtomicPtr<Vec<(JoinHandle<()>, Arc<ThreadUnit<F, N>>)>>>,
    pub(crate) reprt_handler: Arc<AtomicBool>,

    // // list core
    pub(crate) list_core: Arc<ListCore<F>>,
}

impl<F, const N: usize> ThreadUnit<F, N>
where
    F: Fn() + 'static + Send,
{
    pub fn init(
        id: usize,
        total_threads: usize,
        reprt_handler: Arc<AtomicBool>,
        pool: Arc<AtomicPtr<Vec<(JoinHandle<()>, Arc<ThreadUnit<F, N>>)>>>,
        list_core: Arc<ListCore<F>>,
    ) -> Result<ThreadUnit<F, N>, &'static str> {
        let mut queue_vector = Vec::with_capacity(N);
        for _ in 0..N {
            queue_vector.push(AtomicPtr::new(null_mut()));
        }

        let queue_ptr = Box::into_raw(Box::new(
            queue_vector
                .try_into()
                .map_err(|_| "casting vector to array when creating error queue")?,
        ));

        Ok(ThreadUnit {
            id,
            xorshift_seed: AtomicU32::new(1),

            spawn: None,
            running: AtomicPtr::new(null_mut()),

            queue: AtomicPtr::new(queue_ptr),
            batch: N as u32,
            bottom: AtomicUsize::new(0),
            top: AtomicUsize::new(0),

            threads_active: AtomicU64::new(0),
            empty_flag: AtomicBool::new(true),

            reprt_handler,
            pool,
            total_threads,

            list_core,
        })
    }

    fn xorshift(&self) -> u32 {
        let mut x = self.xorshift_seed.load(Ordering::Acquire);
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.xorshift_seed.store(x, Ordering::Release);
        x
    }

    pub fn running(&self) {
        loop {
            // is local queue empty?
            if self.top.load(Ordering::Acquire) >= self.bottom.load(Ordering::Acquire) {
                // empty handling
                // // update flag
                self.empty_flag.store(true, Ordering::SeqCst);

                // // check, any threads have activities on this thread?
                if self.threads_active.load(Ordering::Acquire) > 0 {
                    // activities detected
                    spin_loop();
                };
                // // check representative thread handler
                let is_representative = (*self.reprt_handler).swap(false, Ordering::SeqCst);
                if is_representative {
                    // now, this thread as representative thread
                    // // check primary list
                    if (*self.list_core).is_primary_list_empty() {
                        // empty, swap waiting_task with swap list
                        (*self.list_core).swap_to_primary().unwrap();
                        // check, still empty or not
                        if (*self.list_core).is_primary_list_empty() {
                            // empty, that mean swap list its also empty
                            // release representative thread
                            (*self.reprt_handler).store(true, Ordering::SeqCst);
                            spin_loop();
                            // if no, be steal mode
                            continue;
                        };
                    }
                    // get waiting_task from primary_list
                    let list_waiting_task = (*self.list_core)
                        .get_waiting_task_from_primary_stack::<N>(self.batch)
                        .unwrap();
                    // update local queue
                    // // check twice to ensure, any threads have activities on this thread?
                    if self.threads_active.load(Ordering::Acquire) > 0 {
                        // activities detected
                        spin_loop();
                        continue;
                    };
                    let update_candidate_ptr = Box::into_raw(Box::new(list_waiting_task.list));
                    self.queue.store(update_candidate_ptr, Ordering::Release);

                    // update top and bottom
                    self.top
                        .store(list_waiting_task.top as usize, Ordering::Release);
                    self.bottom
                        .store(list_waiting_task.top as usize, Ordering::Release);

                    // release representative thread
                    (*self.reprt_handler).store(true, Ordering::SeqCst);
                    // update empty_flag
                    self.empty_flag.store(false, Ordering::SeqCst);
                } else {
                    // if no, be steal mode
                    unsafe {
                        let target_thread = loop {
                            // get random id
                            let random = self.xorshift() as usize % self.total_threads;
                            let (_, target_thread) = &(&*self.pool.load(Ordering::Acquire))[random];
                            if target_thread.id == self.id {
                                continue;
                            } else {
                                break target_thread;
                            }
                        };

                        // add activities(knoking the door) to target thread
                        target_thread.threads_active.fetch_add(1, Ordering::Release);
                        // is thread able to steal
                        if target_thread.empty_flag.load(Ordering::SeqCst) {
                            // this target tree empty
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::Release);
                            spin_loop();
                            continue;
                        };

                        // get top and bottom
                        let top = target_thread.top.load(Ordering::Acquire);
                        let bottom = target_thread.bottom.load(Ordering::Acquire);
                        // check
                        if top >= bottom {
                            // this thread literely empty
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::Release);
                            spin_loop();
                            continue;
                        }
                        // get distence
                        let size = bottom - top;
                        if size <= 1 {
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::Release);
                            spin_loop();
                            continue;
                        }
                        // get half
                        let size = size / 2;
                        let new_top = top + size;
                        let status = self.top.compare_exchange(
                            top,
                            new_top,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        );

                        if let Err(_) = status {
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::Release);
                            spin_loop();
                            continue;
                        } else {
                            // validation the task
                            // check empty
                            if target_thread.empty_flag.load(Ordering::SeqCst) {
                                // this target tree empty
                                // close the door
                                target_thread.threads_active.fetch_sub(1, Ordering::Release);
                                spin_loop();
                                continue;
                            };
                            // let queue = vec![];
                            unsafe {
                                for index in top..new_top {
                                    if index >= self.batch as usize {
                                        // out of index
                                        continue;
                                    }

                                    let task = (*target_thread.queue.load(Ordering::Acquire))
                                        [index]
                                        .swap(null_mut(), Ordering::Release);
                                }
                            }
                        }
                    }
                }
            } else {
                // done your work
                unsafe {
                    // get bottom
                    let bottom = self.bottom.load(Ordering::Acquire);
                    // get waiting task
                    let waiting_task = (*self.queue.load(Ordering::Acquire))[bottom]
                        .swap(null_mut(), Ordering::Release);
                    if !waiting_task.is_null() {
                        // running the task
                        let task = Box::from_raw(waiting_task);
                        (task.task)();
                        drop(task);
                    }

                    if bottom == 0 {
                        // empty handling
                        // // update flag
                        self.empty_flag.store(true, Ordering::SeqCst);
                    } else {
                        // next index to top
                        self.bottom.fetch_sub(1, Ordering::Release);
                    }
                }
            }
        }
    }
}
