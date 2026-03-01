use std::sync::atomic::AtomicPtr;

use crate::{
    TaskDependencies, simboli_thread::list_core::wait::dependencies_task::TaskDependenciesCore,
};

pub struct WaitingTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) id: u64,
    pub(crate) task: ExecTask<F, FD, O>,
    pub(crate) next: AtomicPtr<WaitingTask<F, FD, O>>,
    pub(crate) waiting_return_ptr: &'static AtomicPtr<O>,
    // dependencies
    pub(crate) task_dependencies_core_ptr: &'static TaskDependenciesCore<F, O>, // will be shared. to Waiting<O> and WaitingTask<F, O>
    pub(crate) task_dependencies_ptr: &'static TaskDependencies<F, O>,
}

pub trait OutputTrait {}

pub enum ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    Task(F),
    TaskWithDependencies((FD, &'static TaskDependencies<F, FD, O>)),
}

impl<F, FD, O> ExecTask<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub fn exec(self) -> O {
        match self {
            ExecTask::Task(f) => f.exec(),
            ExecTask::TaskWithDependencies((f, dependencies)) => f.exec(dependencies),
        }
    }
}

pub trait TaskTrait<O>
where
    O: OutputTrait,
{
    fn exec(&self) -> O;
}

pub trait TaskWithDependenciesTrait<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, FD, O> + Send + 'static,
    O: OutputTrait + 'static,
{
    fn exec(&self, task_dependencies: &'static TaskDependencies<F, FD, O>) -> O;
}
