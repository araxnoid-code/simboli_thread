fn main() {
    let slot = 2_u32;
    let data = 8_u32;

    let mark = !((1 << slot) - 1);
    let result = data & mark;

    println!("{:032b}", data);
    println!("{:032b}", mark);
    println!("{:032b}", result);
    println!("index: {}", result);
}

fn my_test<F>(f: F)
where
    F: Fn(&i32) + 'static + Send,
{
}
