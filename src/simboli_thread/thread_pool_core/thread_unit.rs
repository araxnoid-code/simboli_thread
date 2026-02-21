use std::{
    hint::spin_loop,
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU32, AtomicU64, AtomicUsize, Ordering},
    },
    thread::{self, JoinHandle},
    time::Duration,
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
    pub(crate) active_counter: Arc<AtomicU64>,
    pub(crate) threads_active: AtomicU64,
    pub(crate) empty_flag: AtomicBool,
    pub(crate) join_flag: Arc<AtomicBool>,

    // share
    // // thread_pool
    pub(crate) total_threads: usize,
    pub(crate) pool: Arc<AtomicPtr<Vec<(Option<JoinHandle<()>>, Arc<ThreadUnit<F, N>>)>>>,
    pub(crate) reprt_handler: Arc<AtomicBool>,

    // // list core
    pub(crate) list_core: Arc<ListCore<F>>,
}

impl<F, const Q: usize> ThreadUnit<F, Q>
where
    F: Fn() + 'static + Send,
{
    pub fn init(
        id: usize,
        total_threads: usize,
        reprt_handler: Arc<AtomicBool>,
        active_counter: Arc<AtomicU64>,
        join_flag: Arc<AtomicBool>,
        pool: Arc<AtomicPtr<Vec<(Option<JoinHandle<()>>, Arc<ThreadUnit<F, Q>>)>>>,
        list_core: Arc<ListCore<F>>,
    ) -> Result<ThreadUnit<F, Q>, &'static str> {
        let mut queue_vector = Vec::with_capacity(Q);
        for _ in 0..Q {
            queue_vector.push(AtomicPtr::new(null_mut()));
        }

        let queue_ptr = Box::into_raw(Box::new(
            queue_vector
                .try_into()
                .map_err(|_| "casting vector to array when creating error queue")?,
        ));
        let local_queue = AtomicPtr::new(queue_ptr);

        Ok(ThreadUnit {
            id,
            xorshift_seed: AtomicU32::new(1),

            spawn: None,
            running: AtomicPtr::new(null_mut()),

            queue: local_queue,
            batch: Q as u32,
            bottom: AtomicUsize::new(0),
            top: AtomicUsize::new(0),

            active_counter: active_counter,
            threads_active: AtomicU64::new(0),
            empty_flag: AtomicBool::new(true),
            join_flag,

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
                // check join
                if self.join_flag.load(Ordering::SeqCst) {
                    break;
                }

                // empty handling
                // // update flag
                let pre_status = self.empty_flag.swap(true, Ordering::SeqCst);
                if !pre_status {
                    let target = !(1_u64 << self.id);
                    self.active_counter.fetch_and(target, Ordering::SeqCst);
                }

                // // check, any threads have activities on this thread?
                if self.threads_active.load(Ordering::SeqCst) > 0 {
                    // activities detected
                    spin_loop();
                    continue;
                };
                // // check representative thread handler
                let is_representative = (*self.reprt_handler).swap(false, Ordering::SeqCst);

                if is_representative {
                    // now, this thread as representative thread
                    // // check primary list
                    if (*self.list_core).is_primary_list_empty() {
                        // empty, swap waiting_task with swap list
                        if let Err(_) = (*self.list_core).swap_to_primary() {
                            // this None, mean empty
                            // release representative thread
                            (*self.reprt_handler).store(true, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        }
                        // check, still empty or not
                        if (*self.list_core).is_primary_list_empty() {
                            // empty, that mean swap list its also empty
                            // release representative thread
                            (*self.reprt_handler).store(true, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        };
                    }
                    // get waiting_task from primary_list
                    let list_waiting_task = if let Ok(list) =
                        (*self.list_core).get_waiting_task_from_primary_stack::<Q>(self.batch)
                    {
                        list
                    } else {
                        (*self.reprt_handler).store(true, Ordering::SeqCst);
                        spin_loop();
                        continue;
                    };

                    // update local queue
                    // // check twice to ensure, any threads have activities on this thread?
                    while self.threads_active.load(Ordering::SeqCst) > 0 {
                        spin_loop();
                        continue;
                    }

                    let update_candidate_ptr = Box::into_raw(Box::new(list_waiting_task.list));
                    let old_addr = self.queue.swap(update_candidate_ptr, Ordering::AcqRel);
                    unsafe {
                        drop(Box::from_raw(old_addr));
                    }

                    // update top and bottom
                    self.top
                        .store(list_waiting_task.top as usize, Ordering::Release);
                    self.bottom
                        .store(list_waiting_task.bottom as usize, Ordering::Release);

                    // release representative thread
                    (*self.reprt_handler).store(true, Ordering::SeqCst);
                    // update empty_flag
                    self.empty_flag.store(false, Ordering::SeqCst);

                    self.active_counter
                        .fetch_or(1_u64 << self.id, Ordering::SeqCst);

                    spin_loop();
                } else {
                    // if no, be steal mode
                    unsafe {
                        let target_thread = loop {
                            // get random id
                            let random = self.xorshift() as usize % self.total_threads;
                            let (_, target_thread) = &(&*self.pool.load(Ordering::Acquire))[random];
                            if target_thread.id == self.id {
                                continue;
                            }
                            break target_thread;
                        };

                        // add activities(knoking the door) to target thread
                        target_thread.threads_active.fetch_add(1, Ordering::SeqCst);
                        // is thread able to steal
                        if target_thread.empty_flag.load(Ordering::SeqCst) {
                            // this target queue empty
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
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
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        }
                        // get distence
                        let size = bottom - top;
                        if size <= 1 {
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        }
                        // get half
                        let size = size / 2;
                        let new_top = top + size;
                        let status = target_thread.top.compare_exchange(
                            top,
                            new_top,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                        );

                        if let Err(_) = status {
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        }
                        // validation the task
                        // check empty
                        if target_thread.empty_flag.load(Ordering::SeqCst) {
                            // this target tree empty
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        };

                        // get task
                        // // scanning start from "end"
                        // // create template
                        let mut list_waiting_task = Vec::new();
                        for _ in 0..Q {
                            list_waiting_task.push(AtomicPtr::new(null_mut()));
                        }

                        let mut list_waiting_task: [AtomicPtr<WaitingTask<F>>; Q] =
                            list_waiting_task.try_into().unwrap();

                        // // check every task
                        let mut out_of_index_counter = false;
                        let mut count = 0;
                        for index in top..new_top {
                            // is out of index?
                            if index >= self.batch as usize {
                                // out of index
                                out_of_index_counter = true;
                                break;
                            }

                            let task = (*target_thread.queue.load(Ordering::Acquire))[index]
                                .swap(null_mut(), Ordering::AcqRel);

                            // is task valid?
                            if task.is_null() {
                                continue;
                            }

                            list_waiting_task[(Q - 1) - count] = AtomicPtr::new(task);

                            count += 1;
                        }

                        // // out of index?
                        if out_of_index_counter {
                            // out of index, mean the range not valid
                            // close the door
                            target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                            spin_loop();
                            continue;
                        }

                        // valid, saving
                        // update local queue
                        // // check to ensure, any threads have activities on this thread?
                        if self.threads_active.load(Ordering::SeqCst) > 0 {
                            // activities detected
                            spin_loop();
                            continue;
                        };
                        let update_candidate_ptr = Box::into_raw(Box::new(list_waiting_task));
                        let old_addr = self.queue.swap(update_candidate_ptr, Ordering::AcqRel);
                        drop(Box::from_raw(old_addr));

                        // update top and bottom
                        self.top.store(Q - count, Ordering::Release);
                        self.bottom.store(Q, Ordering::Release);

                        // update empty_flag
                        self.empty_flag.store(false, Ordering::SeqCst);

                        // close the door
                        target_thread.threads_active.fetch_sub(1, Ordering::SeqCst);
                        self.active_counter
                            .fetch_or(1_u64 << self.id, Ordering::SeqCst);

                        spin_loop();
                    }
                }
            } else {
                // done your work
                unsafe {
                    // get bottom
                    let bottom = self.bottom.load(Ordering::Acquire);
                    if bottom == 0 {
                        // empty handling
                        // // update flag
                        self.empty_flag.store(true, Ordering::SeqCst);
                    }

                    // get waiting task
                    let waiting_task = (*self.queue.load(Ordering::Acquire))[bottom - 1]
                        .swap(null_mut(), Ordering::AcqRel);
                    if !waiting_task.is_null() {
                        // running the task
                        let task = Box::from_raw(waiting_task);
                        // println!("id {} mengeksekusi something", self.id);
                        (task.task)();
                        drop(task);
                    }

                    // next index to top
                    self.bottom.fetch_sub(1, Ordering::Release);
                }
            }
        }
    }
}
