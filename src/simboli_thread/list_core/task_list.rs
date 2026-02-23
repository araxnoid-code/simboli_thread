use std::sync::atomic::AtomicPtr;

use crate::{OutputTrait, TaskTrait, WaitingTask};

pub struct TaskList<F, O, const N: usize>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) list: [AtomicPtr<WaitingTask<F, O>>; N],
    pub(crate) count: u64,
    pub(crate) top: u64,
    pub(crate) bottom: u64,
}
