use std::{
    hint::spin_loop,
    ptr::{self, null_mut},
    sync::atomic::{AtomicPtr, AtomicU64, Ordering},
};

pub struct WaitingTask<F>
where
    F: Fn() + Send + 'static,
{
    id: u64,
    task: F,
    next: AtomicPtr<WaitingTask<F>>,
}

pub struct QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    // primary Stack
    id_counter: AtomicU64,
    start: AtomicPtr<WaitingTask<F>>,
    end: AtomicPtr<WaitingTask<F>>,

    // Swap Stack
    swap_start: AtomicPtr<WaitingTask<F>>,
    swap_end: AtomicPtr<WaitingTask<F>>,
}

pub struct TaskList<F>
where
    F: Fn() + Send + 'static,
{
    start: AtomicPtr<WaitingTask<F>>,
    end: AtomicPtr<WaitingTask<F>>,
    len: usize,
    primary_stack_empty: bool,
}

impl<F> QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    pub fn init() -> QueueCore<F> {
        Self {
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),

            id_counter: AtomicU64::new(0),
            swap_start: AtomicPtr::new(ptr::null_mut()),
            swap_end: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn pop_task_from_primary_stack(&self, len_pop: u32) {
        let start = self.start.load(Ordering::Acquire);
        let end = self.end.load(Ordering::Acquire);

        // scanning
    }

    pub fn swap_to_primary(&self) -> Result<(), &str> {
        let end = self.swap_end.swap(null_mut(), Ordering::AcqRel);
        if !end.is_null() {
            let start = self.swap_start.swap(null_mut(), Ordering::AcqRel);
            self.start.store(start, Ordering::Release);
            self.end.store(end, Ordering::Release);
            Ok(())
        } else {
            Err("SWAP STACK KOSONG")
        }
    }

    pub fn task_from_main_thread(&self, task: F) {
        // main thread only focus in swap queue, base on swap start
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));

        // swap start with new waiting task
        let pre_start_task = self.swap_start.swap(waiting_task_ptr, Ordering::AcqRel);
        if !pre_start_task.is_null() {
            unsafe {
                (*pre_start_task)
                    .next
                    .store(waiting_task_ptr, Ordering::Release);
            }
        } else {
            // saving end waiting task for spanning validation in thread pool later
            self.swap_end.store(waiting_task_ptr, Ordering::Release);
        }
    }
}
