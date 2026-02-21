use std::sync::atomic::AtomicPtr;

use crate::ThreadUnit;

pub struct WaitingTask<F, const Q: usize>
where
    F: Fn(&ThreadUnit<F, Q>) + Send + 'static,
{
    pub(crate) id: u64,
    pub(crate) task: F,
    pub(crate) next: AtomicPtr<WaitingTask<F, Q>>,
}
