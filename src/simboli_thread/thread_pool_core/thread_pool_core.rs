use std::{
    ptr::null_mut,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

use crate::{ListCore, simboli_thread::thread_pool_core::thread_unit::ThreadUnit};

pub struct ThreadPoolCore<F, const N: usize, const Q: usize>
where
    F: Fn() + 'static + Send,
{
    // main thread pool
    pub(crate) queue_size: usize,
    pub(crate) pool: Arc<AtomicPtr<Vec<(Option<JoinHandle<()>>, Arc<ThreadUnit<F, Q>>)>>>,

    // handler
    pub(crate) reprt_handler: Arc<AtomicBool>,
    pub(crate) done_task: Arc<AtomicU64>,
    pub(crate) join_flag: Arc<AtomicBool>,

    // list core
    list_core: Arc<ListCore<F>>,
}

impl<F, const N: usize, const Q: usize> ThreadPoolCore<F, N, Q>
where
    F: Fn() + 'static + Send,
{
    pub fn init(list_core: Arc<ListCore<F>>) -> ThreadPoolCore<F, N, Q> {
        // handler
        let reprt_handler = Arc::new(AtomicBool::new(true));
        let join_flag = Arc::new(AtomicBool::new(false));
        let done_task = Arc::new(AtomicU64::new(0));

        // pool
        let pool = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Vec::with_capacity(
            N,
        )))));

        // group
        // // default for now
        let size = 2;
        let mut reprt_group_handler = Arc::new(AtomicBool::new(true));

        // sync, ensure all threads are initialized before running
        let start_handler = Arc::new(AtomicBool::new(false));
        // MPSC
        let (tx, rx) = mpsc::channel();
        for id in 0..N {
            // MPSC clone
            let tx_clone = tx.clone();

            // pool clone
            let pool_clone = pool.clone();

            // handler clone
            let done_task_clone = done_task.clone();
            let join_flag_clone = join_flag.clone();
            let reprt_handler_clone = reprt_handler.clone();

            // reprt_group_handler, update every 2
            if id % 2 == 0 {
                reprt_group_handler = Arc::new(AtomicBool::new(false));
            }
            let reprt_group_handler_clone = reprt_group_handler.clone();

            // sync clone
            let start_handler_clone = start_handler.clone();

            // list_core clone
            let list_core_clone = list_core.clone();

            // spawn thread
            let spawn = thread::spawn(move || {
                let thread_unit = Arc::new(
                    ThreadUnit::<F, Q>::init(
                        id,
                        N,
                        reprt_handler_clone,
                        join_flag_clone,
                        done_task_clone,
                        pool_clone,
                        list_core_clone,
                        reprt_group_handler_clone,
                    )
                    .unwrap(),
                );

                // give thread to thread pool
                tx_clone.send(thread_unit.clone()).unwrap();

                // waiting
                loop {
                    let start_status = start_handler_clone.load(Ordering::SeqCst);
                    if start_status {
                        break;
                    }
                }

                // running
                thread_unit.running();
            });
            // RX from MPSC
            let shared_thread = rx.recv().unwrap();
            // saving
            let threads_pool = pool.load(std::sync::atomic::Ordering::Acquire);
            unsafe {
                (*threads_pool).push((Some(spawn), shared_thread));
                pool.store(threads_pool, Ordering::Release);
            }
        }

        // start the thread
        start_handler.store(true, Ordering::SeqCst);

        Self {
            list_core: list_core,
            pool: pool,
            queue_size: Q,
            reprt_handler: reprt_handler,
            join_flag,
            done_task,
        }
    }

    /// joining threads in thread pools, does not ensure that all tasks have completed execution before the thread stops
    pub fn join_directly(&self) {
        unsafe {
            self.join_flag.store(true, Ordering::Release);
            for (join_handle, _) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                join_handle.take().unwrap().join().unwrap();
            }

            for (_, thread) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                thread.clean();
            }

            // clean pool
            let pool_ptr = self.pool.swap(null_mut(), Ordering::AcqRel);
            drop(Box::from_raw(pool_ptr));
        }
    }

    /// join threads in thread pools, but ensure all tasks have completed execution before the thread stops
    pub fn join(&self) {
        unsafe {
            // check, all task done
            loop {
                if self.list_core.in_task.load(Ordering::SeqCst)
                    <= self.done_task.load(Ordering::SeqCst)
                {
                    break;
                }
            }

            // join
            self.join_flag.store(true, Ordering::Release);
            for (join_handle, _) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                join_handle.take().unwrap().join().unwrap();
            }

            for (_, thread) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                thread.clean();
            }

            // clean pool
            let pool_ptr = self.pool.swap(null_mut(), Ordering::AcqRel);
            drop(Box::from_raw(pool_ptr));
        }
    }
}
