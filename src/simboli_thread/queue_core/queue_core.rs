use std::sync::atomic::Ordering;

use crate::simboli_thread::queue_core::queue_task::{QueueStack, WaitingTaskStat};

pub struct QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    queue_task_active: u32,
    queue_task_0: QueueStack<F>,
    queue_task_1: QueueStack<F>,
}

impl<F> QueueCore<F>
where
    F: Fn() + Send + 'static,
{
    pub fn init() -> QueueCore<F> {
        Self {
            queue_task_0: QueueStack::init(),
            queue_task_1: QueueStack::init(),
            queue_task_active: 0,
        }
    }

    pub fn toogle_active(&mut self) {
        self.queue_task_active ^= 1;
    }

    pub fn push(&mut self, task: F) {
        let status = if self.queue_task_active == 0 {
            self.queue_task_0.push(task)
        } else {
            self.queue_task_1.push(task)
        };
        self.uncovering_task_handler(status);
    }

    pub fn uncovering_task_handler(&mut self, stat: WaitingTaskStat<F>) {
        if let WaitingTaskStat::Uncovering(task) = stat {
            // use another queue
            self.toogle_active();
            // save in new queue
            if self.queue_task_active == 0 {
                self.queue_task_0.push_after_uncovering_phase(task);
            } else {
                self.queue_task_1.push_after_uncovering_phase(task);
            }
        }
    }

    pub fn show_waiting_task(&self) -> String {
        let top = if self.queue_task_active == 0 {
            &self.queue_task_0.top
        } else {
            &self.queue_task_1.top
        };

        let mut output = String::new();
        if let Ok(top) = top.lock() {
            unsafe {
                let mut node = *Box::from_raw(*top);
                loop {
                    let node_desc = format!("id: {}\n", node.id);
                    output.push_str(&node_desc);

                    if node.next.load(Ordering::SeqCst).is_null() {
                        break;
                    } else {
                        node = *Box::from_raw(node.next.load(Ordering::SeqCst));
                    }
                }
            }
        }

        output
    }
}
