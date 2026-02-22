use std::sync::Arc;

use crate::{ListCore, ThreadPoolCore, simboli_thread::list_core::Waiting};

pub struct SimboliThread<F, T, const N: usize, const Q: usize>
where
    F: Fn() -> T + 'static + Send,
    T: 'static,
{
    // List Core
    list_core: Arc<ListCore<F, T>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, T, N, Q>,
}

impl<F, T, const N: usize, const Q: usize> SimboliThread<F, T, N, Q>
where
    F: Fn() -> T + 'static + Send,
    T: 'static,
{
    pub fn init() -> SimboliThread<F, T, N, Q> {
        let list_core = Arc::new(ListCore::<F, T>::init());
        let thread_pool_core = ThreadPoolCore::<F, T, N, Q>::init(list_core.clone());

        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> Waiting<T> {
        self.list_core.task_from_main_thread(f)
    }

    /// joining threads in thread pools, does not ensure that all tasks have completed execution before the thread stops
    pub fn join_directly(self) {
        self.thread_pool_core.join_directly();
    }

    /// join threads in thread pools, but ensure all tasks have completed execution before the thread stops
    pub fn join(self) {
        self.thread_pool_core.join();
    }
}
