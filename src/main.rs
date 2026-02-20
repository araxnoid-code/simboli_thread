use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, atomic::AtomicPtr},
};

fn main() {
    // test(|| {});
    HashMap::<usize, String>::new();
}

// fn test<F>(f: F)
// where
//     F: Fn() + Debug + Send + 'static,
// {
//     println!("{:?}", f)
// }
