use std::sync::atomic::AtomicPtr;

pub struct WaitingTask<F>
where
    F: Fn() + Send + 'static,
{
    pub(crate) id: u64,
    pub(crate) task: F,
    pub(crate) next: AtomicPtr<WaitingTask<F>>,
}
