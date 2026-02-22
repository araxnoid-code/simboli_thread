use std::{ptr::null_mut, sync::atomic::AtomicPtr};

use simboli_thread::SimboliThread;

#[derive(Debug)]
enum MyOutput {
    number(String),
    Int(i32),
    None,
}

fn main() {
    let thread_pool = SimboliThread::<_, MyOutput, 8, 32>::init();

    let mut out = thread_pool.spawn_task(|| MyOutput::number("hello world".to_string()));

    let my_out = out.block();

    drop(out);

    thread_pool.join();
}
