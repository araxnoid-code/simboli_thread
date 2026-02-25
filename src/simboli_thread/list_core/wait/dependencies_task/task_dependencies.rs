use std::{
    ptr::null_mut,
    sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize},
};

use crate::{OutputTrait, TaskTrait, Waiting, WaitingTask};

// will be shared. to Waiting<O> and WaitingTask<F, O>
pub struct TaskDependenciesCore<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) status: bool,
    pub(crate) activity: AtomicBool,
    pub(crate) counter: AtomicUsize,
    pub(crate) start: AtomicPtr<WaitingTask<F, O>>, // default null, will capture the task need this task output
    pub(crate) end: AtomicPtr<WaitingTask<F, O>>, // default null, will capture the task need this task output
}

impl<F, O> TaskDependenciesCore<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub fn init(counter: usize) -> TaskDependenciesCore<F, O> {
        Self {
            status: true,
            activity: AtomicBool::new(true),
            counter: AtomicUsize::new(counter),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }

    pub fn dummy() -> TaskDependenciesCore<F, O> {
        Self {
            status: false,
            activity: AtomicBool::new(true),
            counter: AtomicUsize::new(0),
            start: AtomicPtr::new(null_mut()),
            end: AtomicPtr::new(null_mut()),
        }
    }
}

pub struct TaskDependencies<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) task_dependencies_ptr: &'static TaskDependenciesCore<F, O>,
    pub waiting_list: Vec<Waiting<O>>,
}

pub trait ArrTaskDependenciesTrait<F, O, const NF: usize>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    fn task_list(self) -> [F; NF];
}
