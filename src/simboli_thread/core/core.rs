use std::sync::Arc;

use crate::{ListCore, ThreadPoolCore};

pub struct SimboliThread<F, const N: usize, const Q: usize>
where
    F: Fn() + 'static + Send,
{
    // List Core
    list_core: Arc<ListCore<F>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, N, Q>,
}

impl<F, const N: usize, const Q: usize> SimboliThread<F, N, Q>
where
    F: Fn() + 'static + Send,
{
    pub fn init() -> SimboliThread<F, N, Q> {
        let list_core = Arc::new(ListCore::<F>::init());
        let thread_pool_core = ThreadPoolCore::<F, N, Q>::init(list_core.clone());

        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) {
        self.list_core.task_from_main_thread(f);
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
