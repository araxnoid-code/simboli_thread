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

// use crate::engine::{InputTrait, InputTraitWithParams, MyEngine, OuputTrait};

// mod engine {
//     use std::marker::PhantomData;

//     pub struct MyEngine<I, O>
//     where
//         I: InputTrait<O>,
//         O: OuputTrait,
//     {
//         input: PhantomData<I>,
//         output: PhantomData<O>,
//     }

//     impl<I, O> MyEngine<I, O>
//     where
//         I: InputTrait<O>,
//         O: OuputTrait,
//     {
//         pub fn init() -> Self {
//             Self {
//                 input: PhantomData::default(),
//                 output: PhantomData::default(),
//             }
//         }

//         pub fn execute(&self, input: I) -> O {
//             input.exec()
//         }

//         pub fn execute_with_params<IP, P>(&self, input: IP, params: P) -> O
//         where
//             IP: InputTraitWithParams<P, O>,
//         {
//             input.exec(params)
//         }
//     }

//     pub trait InputTrait<O>
//     where
//         O: OuputTrait,
//     {
//         fn exec(&self) -> O;
//     }

//     pub trait InputTraitWithParams<P, O>
//     where
//         O: OuputTrait,
//     {
//         fn exec(&self, params: P) -> O;
//     }

//     pub trait OuputTrait {}
// }

// enum MyOuput {
//     String,
//     Int,
//     NIHIL,
// }
// impl OuputTrait for MyOuput {}

// enum WrapperFunction {
//     First(fn() -> MyOuput),
//     Second(fn() -> MyOuput),
//     Third(fn(data: &'static str) -> MyOuput),
// }

// impl InputTrait<MyOuput> for WrapperFunction {
//     fn exec(&self) -> MyOuput {
//         match self {
//             Self::First(f) => f(),
//             Self::Second(f) => f(),
//             _ => MyOuput::NIHIL,
//         }
//     }
// }

// impl InputTraitWithParams<&'static str, MyOuput> for WrapperFunction {
//     fn exec(&self, params: &'static str) -> MyOuput {
//         match self {
//             Self::Third(f) => f(params),
//             _ => MyOuput::NIHIL,
//         }
//     }
// }

// fn main() {
//     let engine = MyEngine::init();

//     let out_1 = engine.execute(WrapperFunction::First(|| {
//         println!("hello");
//         MyOuput::Int
//     }));

//     let out_2 = engine.execute(WrapperFunction::Second(|| {
//         println!("world");
//         MyOuput::String
//     }));

//     let input = "done!";
//     let out_3 = engine.execute_with_params(
//         WrapperFunction::Third(|input: &str| {
//             println!("{}", input);
//             MyOuput::String
//         }),
//         input,
//     );
// }
