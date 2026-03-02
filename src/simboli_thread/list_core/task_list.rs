use std::sync::atomic::AtomicPtr;

use crate::{OutputTrait, TaskTrait, TaskWithDependenciesTrait, WaitingTask};

pub struct TaskList<F, FD, O, const N: usize>
where
    F: TaskTrait<O> + Send + 'static,
    FD: TaskWithDependenciesTrait<O> + Send + 'static,
    O: 'static + OutputTrait + Send,
{
    pub(crate) list: [AtomicPtr<WaitingTask<F, FD, O>>; N],
    pub(crate) count: u64,
    pub(crate) top: u64,
    pub(crate) bottom: u64,
}
