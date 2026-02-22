use std::{
    hint::spin_loop,
    ptr::{self, null_mut},
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
};

use crate::simboli_thread::list_core::{Waiting, task_list::TaskList, waiting_task::WaitingTask};

pub struct ListCore<F, T>
where
    F: Fn() -> T + Send + 'static,
    T: 'static,
{
    // primary Stack
    id_counter: AtomicU64,
    start: AtomicPtr<WaitingTask<F, T>>,
    end: AtomicPtr<WaitingTask<F, T>>,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // Swap Stack
    swap_start: AtomicPtr<WaitingTask<F, T>>,
    swap_end: AtomicPtr<WaitingTask<F, T>>,
}

impl<F, T> ListCore<F, T>
where
    F: Fn() -> T + Send + 'static,
    T: 'static,
{
    pub fn init() -> ListCore<F, T> {
        Self {
            // primary Stack
            id_counter: AtomicU64::new(0),
            start: AtomicPtr::new(ptr::null_mut()),
            end: AtomicPtr::new(ptr::null_mut()),

            // handler
            in_task: Arc::new(AtomicU64::new(0)),

            // Swap Stack
            swap_start: AtomicPtr::new(ptr::null_mut()),
            swap_end: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn is_primary_list_empty(&self) -> bool {
        self.end.load(Ordering::Acquire).is_null()
    }

    pub fn get_waiting_task_from_primary_stack<const N: usize>(
        &self,
        len: u32,
    ) -> Result<TaskList<F, T, N>, &str> {
        let start_waiting_task = self.start.load(Ordering::Acquire);

        // scanning start from "end"
        let mut list_task = Vec::new();
        for _ in 0..N {
            list_task.push(AtomicPtr::new(null_mut()));
        }
        let mut count: u64 = 0;
        unsafe {
            loop {
                let waiting_task = self.end.load(Ordering::Acquire);
                if waiting_task.is_null() {
                    return Err("Primary list empty");
                }

                let next_waiting_task = (*waiting_task).next.load(Ordering::Acquire);
                if next_waiting_task.is_null() {
                    // check, is this last task?
                    if start_waiting_task == waiting_task {
                        // this last task
                        // store the task
                        // // start from bottom
                        list_task[(N - 1) - count as usize] = AtomicPtr::new(waiting_task);
                        // update start
                        self.start.store(null_mut(), Ordering::Release);
                        // update end
                        self.end.store(null_mut(), Ordering::Release);
                        // update counter
                        count += 1;
                        break;
                    } else {
                        // waiting
                        spin_loop();
                        continue;
                    };
                }

                // store the task
                // // start from bottom

                list_task[(N - 1) - count as usize] = AtomicPtr::new(waiting_task);
                // update end
                self.end.store(next_waiting_task, Ordering::Release);
                count += 1;

                if count >= len as u64 {
                    break;
                }
            }
        }

        let list_task = TaskList {
            list: list_task.try_into().unwrap(),
            count,
            top: (N as u64) - count,
            bottom: N as u64,
        };

        Ok(list_task)
    }

    pub fn swap_to_primary(&self) -> Result<(), &str> {
        let end = self.swap_end.swap(null_mut(), Ordering::AcqRel);
        if !end.is_null() {
            let start = self.swap_start.swap(null_mut(), Ordering::AcqRel);
            self.start.store(start, Ordering::Release);
            self.end.store(end, Ordering::Release);
            Ok(())
        } else {
            Err("SWAP STACK EMPTY")
        }
    }

    pub fn task_from_main_thread(&self, task: F) -> Waiting<T> {
        // main thread only focus in swap queue, base on swap start
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::SeqCst);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<T> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
            waiting_return_ptr: return_ptr,
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

        Waiting {
            data_ptr: return_ptr,
            data: None,
        }
    }
}
