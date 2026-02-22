use std::sync::Arc;

use crate::{ListCore, OutputTrait, TaskTrait, ThreadPoolCore, simboli_thread::list_core::Waiting};

pub struct SimboliThread<F, O, const N: usize, const Q: usize>
where
    F: TaskTrait<O> + 'static + Send,
    O: 'static + OutputTrait,
{
    // List Core
    list_core: Arc<ListCore<F, O>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, O, N, Q>,
}

impl<F, O, const N: usize, const Q: usize> SimboliThread<F, O, N, Q>
where
    F: TaskTrait<O> + 'static + Send,
    O: 'static + OutputTrait,
{
    pub fn init() -> SimboliThread<F, O, N, Q> {
        let list_core = Arc::new(ListCore::<F, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, O, N, Q>::init(list_core.clone());

        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> Waiting<O> {
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
