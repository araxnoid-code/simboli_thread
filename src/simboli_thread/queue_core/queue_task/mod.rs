use std::{
    collections::linked_list,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
};

pub enum WaitingTaskStat<F>
where
    F: Fn(),
{
    Done,
    Uncovering(*mut WaitingTask<F>),
}

#[derive(Debug)]
pub struct WaitingTask<F>
where
    F: Fn(),
{
    pub id: u128,
    pub task: F,
    pub next: AtomicPtr<WaitingTask<F>>,
}

#[derive(Debug)]
pub struct QueueStack<F>
where
    F: Fn(),
{
    pub top: Mutex<*mut WaitingTask<F>>,
    pub end: Mutex<*mut WaitingTask<F>>,
    pub task_count: AtomicU64,
}

impl<F> QueueStack<F>
where
    F: Fn(),
{
    pub fn init() -> QueueStack<F> {
        Self {
            top: Mutex::new(ptr::null_mut()),
            end: Mutex::new(ptr::null_mut()),
            task_count: AtomicU64::new(0),
        }
    }

    pub fn push(&self, task: F) -> WaitingTaskStat<F> {
        // create WaitingTask
        let waiting_task = WaitingTask {
            id: uuid::Uuid::new_v4().as_u128(),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
        };
        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        // Locking `end`
        // If `end` unlock, locking `end` then push waiting in linked list
        // If `end` lock, that mean thread pool starting `uncovering phase` and main thread can't waiting for this return WaitingTaskStat::Uncovering
        let mut end_lock = self.end.try_lock();
        if let Ok(end) = end_lock.as_mut() {
            // Main Thread Turn
            if end.is_null() {
                // first waiting_task in list, only update start dan end
                // top_lock save to lock, cause thread pool must check `end` first, if thread pool lock `end`. that block code don't run
                let mut top_lock = self.top.lock().unwrap();
                *top_lock = waiting_task_ptr;
                **end = waiting_task_ptr;
            } else {
                // waiting_task will be continue
                unsafe {
                    // update Previous WaitingTask
                    (*(**end)).next.store(waiting_task_ptr, Ordering::SeqCst);
                    // update end
                    **end = waiting_task_ptr;
                }
            }

            // update task_count
            self.task_count.fetch_add(1, Ordering::SeqCst);

            // return
            WaitingTaskStat::Done
        } else {
            // Main Thread Block, Turn Uncovering `phase`
            WaitingTaskStat::Uncovering(waiting_task_ptr)
        }
    }

    pub fn push_after_uncovering_phase(&self, task: *mut WaitingTask<F>) {
        // after uncovering phase, thread pool will be bussy on another queue
        if let (Ok(end), Ok(top)) = (self.end.lock().as_mut(), self.top.lock().as_mut()) {
            **top = task;
            **end = task;
        }
    }
}
