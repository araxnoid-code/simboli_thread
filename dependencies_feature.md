## Create dependencies and use them
```rust
use std::{thread::sleep, time::Duration};

use simboli_thread::{
    ArrTaskDependenciesTrait, OutputTrait, SimboliThread, TaskTrait, TaskWithDependenciesTrait,
};

enum MyOutput {
    Number(i32),
    String(String),
    None,
}

impl OutputTrait for MyOutput {}

enum MyTask {
    Exec(fn() -> MyOutput),
    ExecWithDependencies(
        fn(dependencies: &'static Vec<simboli_thread::Waiting<MyOutput>>) -> MyOutput,
    ),
}

impl TaskTrait<MyOutput> for MyTask {
    fn exec(&self) -> MyOutput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOutput::None,
        }
    }
}

impl TaskWithDependenciesTrait<MyOutput> for MyTask {
    fn exec(&self, dependencies: &'static Vec<simboli_thread::Waiting<MyOutput>>) -> MyOutput {
        match self {
            MyTask::ExecWithDependencies(f) => f(dependencies),
            _ => MyOutput::None,
        }
    }
}

impl ArrTaskDependenciesTrait<MyTask, MyOutput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}

fn main() {
    let thread_pool = SimboliThread::<MyTask, MyTask, MyOutput, 4, 64>::init();

    let my_dependencies = [
        MyTask::Exec(|| {
            sleep(Duration::from_millis(250));
            println!("task 1 done");
            MyOutput::Number(10)
        }),
        MyTask::Exec(|| {
            sleep(Duration::from_millis(50));
            println!("task 2 done");
            MyOutput::Number(20)
        }),
    ];

    let dependencies = thread_pool.spawn_task_dependencies(my_dependencies);

    // task will be executed after dependencies have been completed
    thread_pool.spawn_task_with_dependencies(
        MyTask::ExecWithDependencies(|dependencies| {
            // access dependencies
            let task_1 = dependencies[0].get().unwrap();
            let task_2 = dependencies[1].get().unwrap();

            println!("task 3 done");
            MyOutput::Number(30)
        }),
        &dependencies,
    );

    thread_pool.join();
}
```
