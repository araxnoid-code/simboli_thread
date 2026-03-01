use std::{thread::sleep, time::Duration};

use simboli_thread::{
    ArrTaskDependenciesTrait, OutputTrait, SimboliThread, TaskDependencies, TaskTrait,
};

#[derive(Debug)]
enum MyOuput {
    String,
    Int,
    None,
}
impl OutputTrait for MyOuput {}

enum MyTask {
    Exec(fn() -> MyOuput),
    WithDependencies(fn(&'static TaskDependencies<MyTask, MyOuput>) -> MyOuput),
}

unsafe impl Send for MyTask {}

impl TaskTrait<MyOuput> for MyTask {
    fn exec(&self) -> MyOuput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOuput::None,
        }
    }

    // fn exec_with_dependencies<F>(
    //     &self,
    //     task_dependecies: &'static TaskDependencies<F, MyOuput>,
    // ) -> MyOuput
    // where
    //     F: TaskTrait<MyOuput> + 'static + Send,
    // {
    //     MyOuput::None
    // }
}

fn main() {
    let thread_pool = SimboliThread::<MyTask, MyOuput, 2, 32>::init();

    let my_dependencies = [
        MyTask::Exec(|| {
            sleep(Duration::from_millis(2000));
            println!("task 1 done");
            MyOuput::Int
        }),
        MyTask::Exec(|| {
            sleep(Duration::from_millis(1000));
            println!("task 2 done");
            MyOuput::String
        }),
    ];

    let waiting_dependencies = thread_pool.spawn_task_dependencies(my_dependencies);

    thread_pool.spawn_task_with_dependencies(
        MyTask::Exec(|| {
            println!("running and done");
            MyOuput::String
        }),
        waiting_dependencies,
    );

    thread_pool.join();
}

impl ArrTaskDependenciesTrait<MyTask, MyOuput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}
