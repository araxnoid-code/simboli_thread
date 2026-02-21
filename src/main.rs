use simboli_thread::{SimboliThread, ThreadUnit};

fn main() {
    // SimboliThread initialization
    // note: SymboliThread manual annotation, namely SymboliThread<fn, number of threads in the thread pool, queue size for each thread in the thread pool>
    //       for the SymboliThread<fn,_,_> part of fn, it's best to leave it to the compiler
    let thread_pool = SimboliThread::<_, 8, 32>::init();

    thread_pool.spawn_task(|thread_unit| {});

    // the main thread will stop here, waiting for all threads to stop and all tasks to be completed
    // thread_pool.join();

    println!("done!")
}

// fn main() {
//     let slot = 2_u32;
//     let data = 8_u32;

//     let mark = !((1 << slot) - 1);
//     let result = data & mark;

//     println!("{:032b}", data);
//     println!("{:032b}", mark);
//     println!("{:032b}", result);
//     println!("index: {}", result);
// }

// fn my_test<F>(f: F)
// where
//     F: Fn(&i32) + 'static + Send,
// {
// }
