use std::sync::atomic::AtomicPtr;

use crate::{
    TaskDependencies, Waiting,
    simboli_thread::list_core::wait::dependencies_task::TaskDependenciesCore,
};

pub struct WaitingTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) id: u64,
    pub(crate) task: ExecTask<F, FD, O>,
    pub(crate) next: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) waiting_return_ptr: &'static AtomicPtr<O>,
    // dependencies
    pub(crate) task_dependencies_core_ptr: &'static TaskDependenciesCore<F, FD, O>, // will be shared. to Waiting<O> and WaitingTask<F, O>
    pub(crate) task_dependencies_ptr: &'static Vec<Waiting<O>>,
}

pub trait OutputTrait {}

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    Task(F),
    TaskWithDependencies(FD),
    _Output(O),
}

pub trait TaskTrait<O>
where
    O: OutputTrait,
{
    fn exec(&self) -> O;

    fn is_with_dependencies(&self) -> bool {
        false
    }
}

pub trait TaskWithDependenciesTrait<O>
where
    O: OutputTrait + 'static + Send,
{
    fn exec(&self, dependencies: &'static Vec<Waiting<O>>) -> O;

    fn is_with_dependencies(&self) -> bool {
        true
    }
}
