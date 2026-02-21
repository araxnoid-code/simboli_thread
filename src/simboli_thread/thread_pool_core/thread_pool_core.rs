use std::{
    fmt::Debug,
    hint::spin_loop,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicPtr, AtomicU64, AtomicUsize, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{ListCore, simboli_thread::thread_pool_core::thread_unit::ThreadUnit};

pub struct ThreadPoolCore<F, const N: usize, const Q: usize>
where
    F: Fn() + 'static + Send,
{
    // thread pool
    pub(crate) reprt_handler: Arc<AtomicBool>,
    pub(crate) active_counter: Arc<AtomicU64>,
    pub(crate) queue_size: usize,
    pub(crate) pool: Arc<AtomicPtr<Vec<(Option<JoinHandle<()>>, Arc<ThreadUnit<F, Q>>)>>>,
    pub(crate) join_flag: Arc<AtomicBool>,

    // list core
    list_core: Arc<ListCore<F>>,
}

impl<F, const N: usize, const Q: usize> ThreadPoolCore<F, N, Q>
where
    F: Fn() + 'static + Send,
{
    pub fn init(list_core: Arc<ListCore<F>>) -> ThreadPoolCore<F, N, Q> {
        let reprt_handler = Arc::new(AtomicBool::new(true));
        let pool = Arc::new(AtomicPtr::new(Box::into_raw(Box::new(Vec::with_capacity(
            N,
        )))));
        let join_flag = Arc::new(AtomicBool::new(false));
        let active_counter = Arc::new(AtomicU64::new((1 << N) - 1));

        let start_handler = Arc::new(AtomicBool::new(false));

        let (tx, rx) = mpsc::channel();
        for id in 0..N {
            let tx_clone = tx.clone();
            let start_handler_clone = start_handler.clone();

            let pool_clone = pool.clone();
            let active_counter_clone = active_counter.clone();
            let join_flag_clone = join_flag.clone();
            let list_core_clone = list_core.clone();
            let share_reprt_handler = reprt_handler.clone();
            let spawn = thread::spawn(move || {
                // let id_spawn = id;
                let thread_unit = Arc::new(
                    ThreadUnit::<F, Q>::init(
                        id,
                        N,
                        share_reprt_handler,
                        active_counter_clone,
                        join_flag_clone,
                        pool_clone,
                        list_core_clone,
                    )
                    .unwrap(),
                );

                tx_clone.send(thread_unit.clone()).unwrap();

                loop {
                    let start_status = start_handler_clone.load(Ordering::SeqCst);
                    if start_status {
                        break;
                    }
                }

                println!("thread {} runnning", id);
                println!();
                thread_unit.running();
            });

            let shared_thread = rx.recv().unwrap();
            let threads_pool = pool.load(std::sync::atomic::Ordering::Acquire);
            unsafe {
                (*threads_pool).push((Some(spawn), shared_thread));
                pool.store(threads_pool, Ordering::Release);
            }
        }

        start_handler.store(true, Ordering::SeqCst);

        Self {
            list_core: list_core,
            pool: pool,
            active_counter,
            queue_size: Q,
            reprt_handler: reprt_handler,
            join_flag,
        }
    }

    pub fn join(&self) {
        let mut count_counter = 0;
        loop {
            let counter = (*self.active_counter).load(Ordering::SeqCst);
            if counter == 0 && count_counter > 5 {
                break;
            } else if counter == 0 {
                count_counter += 1;
            }
            spin_loop();
        }
        unsafe {
            self.join_flag.store(true, Ordering::Release);
            for (join_handle, _) in (*self.pool.load(Ordering::Acquire)).iter_mut() {
                join_handle.take().unwrap().join().unwrap();
            }
        }
    }
}
