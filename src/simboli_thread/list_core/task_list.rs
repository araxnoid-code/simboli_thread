use std::sync::atomic::AtomicPtr;

use crate::simboli_thread::list_core::waiting_task::WaitingTask;

pub struct TaskList<F>
where
    F: Fn() + Send + 'static,
{
    pub(crate) list: Vec<AtomicPtr<WaitingTask<F>>>,
    pub(crate) count: u64,
}
