use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize},
};

use crate::{
    OutputTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask,
    simboli_thread::list_core::Waiting,
};

// will be shared. to Waiting<O> and WaitingTask<F, O>
pub struct TaskDependenciesCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) status: bool,
    pub(crate) done: AtomicBool,
    pub(crate) counter: AtomicUsize,
    pub(crate) start: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
    pub(crate) end: AtomicPtr<WaitingTask<F, FD, O>>, // default null, will capture the task need this task output
}

impl<F, FD, O> TaskDependenciesCore<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub fn init(counter: usize) -> TaskDependenciesCore<F, FD, O> {
        Self {
            status: true,
            done: AtomicBool::new(false),
            counter: AtomicUsize::new(counter),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }

    pub fn blank() -> TaskDependenciesCore<F, FD, O> {
        Self {
            status: false,
            done: AtomicBool::new(false),
            counter: AtomicUsize::new(0),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }
}

pub struct TaskDependencies<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) task_dependencies_ptr: &'static TaskDependenciesCore<F, FD, O>,
    pub waiting_list: Vec<Waiting<O>>,
}

impl<F, FD, O> TaskDependencies<F, FD, O>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<F, O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub fn blank() -> TaskDependencies<F, FD, O> {
        Self {
            task_dependencies_ptr: Box::leak(Box::new(TaskDependenciesCore::blank())),
            waiting_list: vec![],
        }
    }
}

pub trait ArrTaskDependenciesTrait<F, O, const NF: usize>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    fn task_list(self) -> [F; NF];
}
