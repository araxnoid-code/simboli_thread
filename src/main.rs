use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        Arc, Mutex,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
    thread,
    time::{self, Duration, UNIX_EPOCH},
};

use simboli_thread::SimboliThread;

fn main() {
    // let tick = time::SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_millis();

    let simboli_thread = SimboliThread::<_, 8, 32>::init();

    let counter = Arc::new(AtomicUsize::new(0));
    for i in 0..1000 {
        let counter_clone = counter.clone();
        simboli_thread.spawn_task(move || {
            thread::sleep(Duration::from_millis(25));
            let total = counter_clone.fetch_add(1, Ordering::SeqCst);
        });
    }

    simboli_thread.join_directly();

    // let tock = time::SystemTime::now()
    //     .duration_since(UNIX_EPOCH)
    //     .unwrap()
    //     .as_millis();

    // println!(
    //     "done with {} and {}ms",
    //     counter.load(Ordering::SeqCst),
    //     tock - tick
    // );
}
