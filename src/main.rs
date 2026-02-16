use std::sync::Arc;

use simboli_thread::{self, QueueCore};

fn main() {
    let mut queue_core = QueueCore::<fn()>::init();
    // queue_core.push(|| {});
    // queue_core.push(|| {});
    // queue_core.push(|| {});
    // queue_core.push(|| {});
    // queue_core.push(|| {});

    // let list = queue_core.show_waiting_task();
    // println!("{}", list);
}
