use std::sync::{Arc, atomic::AtomicPtr};

pub struct WaitingTask<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub(crate) id: u64,
    pub(crate) task: F,
    pub(crate) next: AtomicPtr<WaitingTask<F, O>>,
    pub(crate) waiting_return_ptr: &'static AtomicPtr<O>,
}

pub trait OutputTrait {}

pub trait TaskTrait<O>
where
    O: OutputTrait,
{
    fn exec(&self) -> O;
}
