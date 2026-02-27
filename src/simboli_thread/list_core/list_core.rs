use std::{
    hint::spin_loop,
    ptr::{self, null_mut},
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicU64, Ordering},
    },
};

use crate::{
    TaskDependencies,
    simboli_thread::list_core::{
        ArrTaskDependenciesTrait, OutputTrait, TaskDependenciesCore, TaskTrait, Waiting,
        WaitingTask, task_list::TaskList,
    },
};

pub struct ListCore<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    // primary Stack
    id_counter: AtomicU64,
    start: AtomicPtr<WaitingTask<F, O>>,
    end: AtomicPtr<WaitingTask<F, O>>,

    // handler
    pub(crate) in_task: Arc<AtomicU64>,

    // Swap Stack
    swap_start: AtomicPtr<WaitingTask<F, O>>,
    swap_end: AtomicPtr<WaitingTask<F, O>>,
}

impl<F, O> ListCore<F, O>
where
    F: TaskTrait<O> + Send + 'static,
    O: 'static + OutputTrait,
{
    pub fn init() -> ListCore<F, O> {
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
    ) -> Result<TaskList<F, O, N>, &str> {
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

    pub fn insert_list_from_harvesting(
        &self,
        harvesting_start: AtomicPtr<WaitingTask<F, O>>,
        harvesting_end: AtomicPtr<WaitingTask<F, O>>,
    ) {
        // insert_list_from_harvesting, must not null
        let harvesting_end = harvesting_end.swap(null_mut(), Ordering::AcqRel);
        if !harvesting_end.is_null() {
            let harvesting_start = harvesting_start.swap(null_mut(), Ordering::AcqRel);
            let prev_start = self.start.swap(harvesting_start, Ordering::AcqRel);
            if !prev_start.is_null() {
                unsafe {
                    (*prev_start).next.store(harvesting_end, Ordering::Release);
                }
            } else {
                self.end.store(harvesting_end, Ordering::Release);
            }
        }
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

    pub fn spawn_task_with_dependencies(
        &self,
        task: F,
        dependencies: TaskDependencies<F, O>,
    ) -> Waiting<O> {
        // main thread only focus in swap queue, base on swap start
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::SeqCst);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
            waiting_return_ptr: return_ptr,
            task_dependencies_ptr: Box::leak(Box::new(TaskDependenciesCore::dummy())),
        };

        let waiting_task_ptr = Box::into_raw(Box::new(waiting_task));
        // check depencies
        if !dependencies
            .task_dependencies_ptr
            .done
            .load(Ordering::SeqCst)
        {
            // insert into depencies waiting

            let status = dependencies.task_dependencies_ptr.start.compare_exchange(
                dependencies
                    .task_dependencies_ptr
                    .start
                    .load(Ordering::Acquire),
                waiting_task_ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            );

            if let Ok(prev_waiting_task) = status {
                if prev_waiting_task.is_null() {
                    // chek again
                    if !dependencies
                        .task_dependencies_ptr
                        .done
                        .load(Ordering::SeqCst)
                    {
                        if !prev_waiting_task.is_null() {
                            unsafe {
                                (*prev_waiting_task)
                                    .next
                                    .store(waiting_task_ptr, Ordering::Release);
                            }
                        } else {
                            dependencies
                                .task_dependencies_ptr
                                .end
                                .store(waiting_task_ptr, Ordering::Release);
                        }
                    } else {
                        dependencies
                            .task_dependencies_ptr
                            .end
                            .store(null_mut(), Ordering::Release);
                        dependencies
                            .task_dependencies_ptr
                            .start
                            .store(null_mut(), Ordering::Release);
                        self.spawn_task_with_dependencies_normal(waiting_task_ptr, return_ptr);
                    };
                } else {
                    if !prev_waiting_task.is_null() {
                        unsafe {
                            (*prev_waiting_task)
                                .next
                                .store(waiting_task_ptr, Ordering::Release);
                        }
                    } else {
                        dependencies
                            .task_dependencies_ptr
                            .end
                            .store(waiting_task_ptr, Ordering::Release);
                    }
                }
            } else {
                self.spawn_task_with_dependencies_normal(waiting_task_ptr, return_ptr);
            }
        } else {
            self.spawn_task_with_dependencies_normal(waiting_task_ptr, return_ptr);
        };

        Waiting {
            data_ptr: return_ptr,
            data: None,
        }
    }

    fn spawn_task_with_dependencies_normal(
        &self,
        waiting_task_ptr: *mut WaitingTask<F, O>,
        return_ptr: &'static AtomicPtr<O>,
    ) {
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

    pub fn spawn_task_dependencies<D, const NF: usize>(
        &self,
        dependencies: D,
    ) -> TaskDependencies<F, O>
    where
        D: ArrTaskDependenciesTrait<F, O, NF>,
    {
        // create dependencies
        let task_dependencies_ptr: &'static TaskDependenciesCore<F, O> =
            Box::leak(Box::new(TaskDependenciesCore::init(NF)));

        // output
        let mut waiting_output = Vec::with_capacity(NF);

        // task_dependencies
        for task in dependencies.task_list() {
            // update in_task handler
            self.in_task.fetch_add(1, Ordering::SeqCst);
            // create return_ptr
            let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));

            // create waiting task
            let waiting_task = WaitingTask {
                id: self.id_counter.fetch_add(1, Ordering::Release),
                task,
                next: AtomicPtr::new(ptr::null_mut()),
                waiting_return_ptr: return_ptr,
                task_dependencies_ptr: task_dependencies_ptr,
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

            waiting_output.push(Waiting {
                data_ptr: return_ptr,
                data: None,
            });
        }

        TaskDependencies {
            waiting_list: waiting_output,
            task_dependencies_ptr: task_dependencies_ptr,
        }
    }

    pub fn spawn_task(&self, task: F) -> Waiting<O> {
        // main thread only focus in swap queue, base on swap start
        // update in_task handler
        self.in_task.fetch_add(1, Ordering::SeqCst);
        // create return_ptr
        let return_ptr: &'static AtomicPtr<O> = Box::leak(Box::new(AtomicPtr::new(null_mut())));
        // create waiting task
        let waiting_task = WaitingTask {
            id: self.id_counter.fetch_add(1, Ordering::Release),
            task,
            next: AtomicPtr::new(ptr::null_mut()),
            waiting_return_ptr: return_ptr,
            task_dependencies_ptr: Box::leak(Box::new(TaskDependenciesCore::dummy())),
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
