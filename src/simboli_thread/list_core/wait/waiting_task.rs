use std::sync::atomic::AtomicPtr;

use crate::{
    TaskDependencies, simboli_thread::list_core::wait::dependencies_task::TaskDependenciesCore,
};

pub struct WaitingTask<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) id: u64,
    pub(crate) task: F,
    pub(crate) next: AtomicPtr<WaitingTask<F, O>>,
    pub(crate) waiting_return_ptr: &'static AtomicPtr<O>,
    // dependencies
    pub(crate) task_dependencies_core_ptr: &'static TaskDependenciesCore<F, O>, // will be shared. to Waiting<O> and WaitingTask<F, O>
    pub(crate) task_dependencies_ptr: &'static TaskDependencies<F, O>,
}

pub trait OutputTrait {}

pub enum ExecTask<F, TD> {
    Task(F),
    TaskWithDependencies(TD),
}

pub trait TaskTrait<O>
where
    O: OutputTrait,
{
    fn exec(&self) -> O;
}
