use std::{
    fmt::Debug,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicPtr, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
};

use crate::{ListCore, simboli_thread::thread_pool_core::thread_unit::ThreadUnit};

pub struct ThreadPoolCore<F, const N: usize>
where
    F: Fn() + 'static + Send,
{
    // thread pool
    pub(crate) reprt_handler: Arc<AtomicBool>,
    pub(crate) pool: Vec<(JoinHandle<()>, ThreadUnit<F, N>)>,

    // list core
    list_core: Arc<ListCore<F>>,
}

impl<F, const N: usize> ThreadPoolCore<F, N>
where
    F: Fn() + 'static + Send,
{
    pub fn init(list_core: Arc<ListCore<F>>, threads: usize) {
        let reprt_handler = Arc::new(AtomicBool::new(true));
        let pool = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Vec::with_capacity(
            threads,
        )))));

        let (tx, rx) = mpsc::channel();
        for id in 0..threads {
            let tx_clone = tx.clone();

            let pool_clone = pool.clone();
            let list_core_clone = list_core.clone();
            let share_reprt_handler = reprt_handler.clone();
            let spawn = thread::spawn(move || {
                // let id_spawn = id;
                let thread_unit = Arc::new(
                    ThreadUnit::<F, N>::init(
                        id,
                        threads,
                        share_reprt_handler,
                        pool_clone,
                        list_core_clone,
                    )
                    .unwrap(),
                );

                tx_clone.send(thread_unit.clone()).unwrap();
            });

            let shared_thread = rx.recv().unwrap();
            let threads_pool = pool.load(std::sync::atomic::Ordering::Acquire);
            unsafe {
                (*threads_pool).push((spawn, shared_thread));
            }
            pool.store(threads_pool, Ordering::Release);
        }
    }
}
