use std::sync::Arc;

use crate::simboli_thread::thread_pool_core::thread_unit::ThreadUnit;

pub struct ThreadPatemeter<F, const Q: usize>
where
    F: Fn() + 'static + Send,
{
    pub thread: Arc<ThreadUnit<F, Q>>,
}
