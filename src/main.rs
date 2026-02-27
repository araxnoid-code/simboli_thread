use std::{thread::sleep, time::Duration};

use simboli_thread::{ArrTaskDependenciesTrait, OutputTrait, SimboliThread, TaskTrait};

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
    let thread_pool = SimboliThread::<MyTask, MyOuput, 2, 32>::init();

    let my_dependencies = [
        MyTask(|| {
            println!("task 1 done");
            MyOuput::Int
        }),
        MyTask(|| {
            eprintln!("task 2 done");
            MyOuput::String
        }),
    ];

    let waiting_dependencies = thread_pool.spawn_task_dependencies(my_dependencies);

    thread_pool.spawn_task_with_dependencies(
        MyTask(|| {
            println!("running and done");
            MyOuput::String
        }),
        waiting_dependencies,
    );

    // loop {}
    thread_pool.join();
    // let output = waiting_dependencies.waiting_list.;

    // let array: [i32; 10];
    // println!("{:?}", array);
}

impl ArrTaskDependenciesTrait<MyTask, MyOuput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}
