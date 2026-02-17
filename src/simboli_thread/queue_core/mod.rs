use std::{
    ptr,
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

pub struct WaitingTask<F>
where
    F: Fn() + Send + 'static,
{
    id: u128,
    task: F,
    next: AtomicPtr<WaitingTask<F>>,
}

pub struct QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    // linked_list
    start: AtomicPtr<WaitingTask<F>>,
    end: AtomicPtr<WaitingTask<F>>,
    len: AtomicU64,

    // swap phase
    swap_start: AtomicPtr<WaitingTask<F>>,
    swap_len: AtomicU64,
}

impl<F> QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    pub fn init() -> QueueCore<F> {
        Self {
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),
            len: AtomicU64::new(0),

            swap_start: AtomicPtr::new(ptr::null_mut()),
            swap_len: AtomicU64::new(0),
        }
    }

    pub fn task_from_main_thread(&self, task: F) {
        // main thread only focus in swap queue, base on swap start
        // create waiting task
        let waiting_task = WaitingTask {
            id: uuid::Uuid::new_v4().as_u128(),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        if self.swap_start.load(Ordering::Acquire).is_null() {
            // start is null, list empty
            // update swap start
            self.swap_start.store(waiting_task_ptr, Ordering::Release);
        } else {
            // continue the list
            // update swap start with new waiting task
            unsafe {
                // swap start with new waiting task
                let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
                // update waiting task with previous start
                (*waiting_task_ptr)
                    .next
                    .store(pre_start_task, Ordering::Release);
            }
        }
    }
}
