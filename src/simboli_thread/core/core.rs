use std::sync::Arc;

use crate::{
    ArrTaskDependenciesWithDependenciesTrait, ListCore, OutputTrait, TaskDependencies, TaskTrait,
    TaskWithDependenciesTrait, ThreadPoolCore,
    simboli_thread::list_core::{ArrTaskDependenciesTrait, Waiting},
};

pub struct SimboliThread<F, FD, O, const N: usize, const Q: usize>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send + Send,
{
    // List Core
    list_core: Arc<ListCore<F, FD, O>>,
    // thread pool Core
    thread_pool_core: ThreadPoolCore<F, FD, O, N, Q>,
}

impl<F, FD, O, const N: usize, const Q: usize> SimboliThread<F, FD, O, N, Q>
where
    F: TaskTrait<O> + 'static + Send,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub fn init() -> SimboliThread<F, FD, O, N, Q> {
        let list_core = Arc::new(ListCore::<F, FD, O>::init());
        let thread_pool_core = ThreadPoolCore::<F, FD, O, N, Q>::init(list_core.clone());
        Self {
            list_core,
            thread_pool_core,
        }
    }

    pub fn spawn_task(&self, f: F) -> Waiting<O> {
        self.list_core.spawn_task(f)
    }

    pub fn spawn_task_dependencies_with_dependencies<D, const NF: usize>(
        &self,
        dependencies: D,
        with_dependencies: &TaskDependencies<F, FD, O>,
    ) -> TaskDependencies<F, FD, O>
    where
        D: ArrTaskDependenciesWithDependenciesTrait<FD, O, NF>,
    {
        self.list_core
            .spawn_task_dependencies_with_dependencies(dependencies, &with_dependencies)
    }

    pub fn spawn_task_dependencies<D, const NF: usize>(
        &self,
        dependencies: D,
    ) -> TaskDependencies<F, FD, O>
    where
        D: ArrTaskDependenciesTrait<F, O, NF>,
    {
        self.list_core.spawn_task_dependencies(dependencies)
    }

    pub fn spawn_task_with_dependencies(
        &self,
        task: FD,
        dependencies: &TaskDependencies<F, FD, O>,
    ) -> Waiting<O> {
        self.list_core
            .spawn_task_with_dependencies(task, dependencies, None)
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
