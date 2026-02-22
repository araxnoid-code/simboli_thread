use std::sync::{Arc, atomic::AtomicPtr};

pub struct WaitingTask<F, T>
where
    F: Fn() -> T + Send + 'static,
    T: 'static,
{
    pub(crate) id: u64,
    pub(crate) task: F,
    pub(crate) next: AtomicPtr<WaitingTask<F, T>>,
    pub(crate) waiting_return_ptr: &'static AtomicPtr<T>,
}
