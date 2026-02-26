use std::{thread::sleep, time::Duration};

use simboli_thread::{
    ArrTaskDependenciesTrait, OutputTrait, SimboliThread, SpwanTaskWithDependenciesTrait, TaskTrait,
};

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
    let value = 4_u64;
    let marking = !((1_u64 << 1) - 1);
    let result = value & marking;

    println!("{:08b}", value);
    println!("{:08b}", marking);
    println!("{:08b} => {}", result, result);

    let thread_pool = SimboliThread::<MyTask, MyOuput, 8, 32>::init();

    let my_dependencies = [MyTask(|| MyOuput::Int), MyTask(|| MyOuput::String)];
    let my_waiting = thread_pool.spawn_task_dependencies(my_dependencies);

    // let array: [i32; 10];
    // println!("{:?}", array);
}

impl ArrTaskDependenciesTrait<MyTask, MyOuput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}

impl SpwanTaskWithDependenciesTrait<MyTask, MyOuput, 1> for [MyTask; 1] {
    fn task(self) -> [MyTask; 1] {
        self
    }
}
