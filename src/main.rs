use std::{thread::sleep, time::Duration};

use simboli_thread::{OutputTrait, SimboliThread, TaskTrait};

#[derive(Debug)]
enum MyOuput {
    String,
    Int,
}
impl OutputTrait for MyOuput {}

struct MyTask(fn() -> MyOuput);

impl TaskTrait<MyOuput> for MyTask {
    fn exec(&self) -> MyOuput {
        (self.0)()
    }
}

fn main() {
    let thread_pool = SimboliThread::<MyTask, MyOuput, 8, 32>::init();

    let wait_a = thread_pool.spawn_task(MyTask(|| {
        sleep(Duration::from_millis(500));
        println!("hello world");
        MyOuput::String
    }));

    let wait_b = thread_pool.spawn_task(MyTask(|| {
        sleep(Duration::from_millis(500));
        println!("this different task!");
        MyOuput::Int
    }));

    let wait_c = thread_pool.spawn_task(MyTask(|| {
        sleep(Duration::from_millis(500));
        println!("that's work!");
        MyOuput::Int
    }));

    thread_pool.join();

    println!("done!")
}
