use std::{thread::sleep, time::Duration};

use simboli_thread::{
    ArrTaskDependenciesTrait, ArrTaskDependenciesWithDependenciesTrait, OutputTrait, SimboliThread,
    TaskTrait, TaskWithDependenciesTrait, Waiting,
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
    WithDependencies(fn(&'static Vec<Waiting<MyOuput>>) -> MyOuput),
}

impl TaskTrait<MyOuput> for MyTask {
    fn exec(&self) -> MyOuput {
        match self {
            MyTask::Exec(f) => f(),
            _ => MyOuput::None,
        }
    }
}

impl TaskWithDependenciesTrait<MyOuput> for MyTask {
    fn exec(&self, dependencies: &'static Vec<Waiting<MyOuput>>) -> MyOuput {
        match self {
            MyTask::WithDependencies(f) => f(dependencies),
            MyTask::Exec(f) => f(),
        }
    }

    fn is_with_dependencies(&self) -> bool {
        match self {
            MyTask::Exec(_) => false,
            MyTask::WithDependencies(_) => true,
        }
    }
}

fn main() {
    let thread_pool = SimboliThread::<MyTask, MyTask, MyOuput, 10, 1024>::init();

    for i in 0..1000 {
        let my_dependencies = [
            MyTask::Exec(|| {
                sleep(Duration::from_millis(100));

                MyOuput::Int
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(50));

                MyOuput::String
            }),
        ];

        let dependencies_1 = thread_pool.spawn_task_dependencies(my_dependencies);

        let other_dependencies = [
            MyTask::WithDependencies(|dependencies| {
                sleep(Duration::from_millis(100));
                let task_1 = &dependencies[0].get().unwrap();

                let task_2 = dependencies[1].get().unwrap();

                MyOuput::String
            }),
            MyTask::Exec(|| {
                sleep(Duration::from_millis(10));
                MyOuput::String
            }),
        ];

        let dependencies_2 = thread_pool
            .spawn_task_dependencies_with_dependencies(other_dependencies, &dependencies_1);

        thread_pool.spawn_task_with_dependencies(
            MyTask::WithDependencies(|dependencies| {
                let task_1 = &dependencies[0].get();
                let task_2 = &dependencies[1].get();
                MyOuput::String
            }),
            &dependencies_2,
        );
    }

    thread_pool.join();
}

impl ArrTaskDependenciesTrait<MyTask, MyOuput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}

impl ArrTaskDependenciesWithDependenciesTrait<MyTask, MyOuput, 2> for [MyTask; 2] {
    fn task_list(self) -> [MyTask; 2] {
        self
    }
}
