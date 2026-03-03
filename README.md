<div align="center">
    <h1>simboli_thread</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>Version / 0.0.2</p>
    </b>
</div>

## About
`simboli_thread`, thread pool management written in rust.

## Warning
there is still a memory leak occurring and unstable.

## Changelog
[changelog.md](https://github.com/araxnoid-code/simboli_thread/blob/main/changelog.md)

## Starting
### Installation
Run the following Cargo command in your project directory:
```sh
cargo add simboli_thread
```
Or add the following line to your Cargo.toml:
```toml
simboli_thread = "0.0.2"
```

### Code
```rust
use std::{thread::sleep, time::Duration};
use simboli_thread::{OutputTrait, SimboliThread, TaskTrait, TaskWithDependenciesTrait};
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

fn main() {
    let thread_pool = SimboliThread::<MyTask, MyTask, MyOutput, 4, 64>::init();

    let task_1 = thread_pool.spawn_task(MyTask::Exec(|| {
        sleep(Duration::from_millis(100));
        MyOutput::Number(10)
    }));

    let task_2 = thread_pool.spawn_task(MyTask::Exec(|| {
        sleep(Duration::from_millis(100));
        MyOutput::String("done".to_string())
    }));

    // blocking
    let task_1 = task_1.block();
    if let Some(MyOutput::Number(number)) = task_1 {
        println!("task_1 : {}", number)
    }

    let task_2 = task_2.block();
    if let Some(MyOutput::String(str)) = task_2 {
        println!("task_2 : {}", str)
    }

    thread_pool.join();
}
```
