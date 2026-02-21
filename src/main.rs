use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
    thread,
    time::Duration,
};

use simboli_thread::{SimboliThread, my_test};

fn main() {
    let simboli_thread = SimboliThread::<_, 8, 32>::init();

    let counter = Arc::new(Mutex::new(0));
    for i in 0..1000 {
        let counter_clone = counter.clone();
        simboli_thread.spawn_task(move || {
            // if let Ok(lock) = counter_clone.lock().as_mut() {
            // **lock += 1;
            // println!("done counter => {}", lock);
            // }
            // println!("task {} done", i);
        });
    }

    // println!("done");

    loop {}
    // simboli_thread.join();
}
