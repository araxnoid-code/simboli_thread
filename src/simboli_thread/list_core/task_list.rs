use std::sync::atomic::AtomicPtr;

use crate::simboli_thread::list_core::waiting_task::WaitingTask;

pub struct TaskList<F, const N: usize>
where
    F: Fn() + Send + 'static,
{
    pub(crate) list: [AtomicPtr<WaitingTask<F>>; N],
    pub(crate) count: u64,
    pub(crate) top: u64,
    pub(crate) bottom: u64,
}
