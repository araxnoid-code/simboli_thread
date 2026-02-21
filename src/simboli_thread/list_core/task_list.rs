use std::sync::atomic::AtomicPtr;

use crate::{ThreadUnit, simboli_thread::list_core::waiting_task::WaitingTask};

pub struct TaskList<F, const N: usize, const Q: usize>
where
    F: Fn(&ThreadUnit<F, Q>) + Send + 'static,
{
    pub(crate) list: [AtomicPtr<WaitingTask<F, Q>>; N],
    pub(crate) count: u64,
    pub(crate) top: u64,
    pub(crate) bottom: u64,
}
