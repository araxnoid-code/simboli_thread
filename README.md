<div align="center">
    <h1>simboli_thread</h1>
    <b><p>Thread Pool Management</p></b>
    <p>⚙️ under development ⚙️</p>
    <b>
        <p>Version / 0.0.1</p>
    </b>
</div>

## About
`simboli_thread`, thread pool management yang ditulis di rust.

## Warning
ini hanyalah project coba coba, kemungkinan besar akan banyak bug dan juga mungkin saja tidak akan ada update kedepannya.

## Main Concept
konsep konsep utama yang digunakan pada `simboli_thread`

[main_concept.md](./main_concept.md)

## Starting
### Installation
Run the following Cargo command in your project directory:
```sh
cargo add simboli_thread
```
Or add the following line to your Cargo.toml:
```toml
simboli_thread = "0.0.1"
```

### Code
```rust
use simboli_thread::SimboliThread;

fn main() {
    // SimboliThread initialization
    // note: SymboliThread manual annotation, namely SymboliThread<fn, number of threads in the thread pool, queue size for each thread in the thread pool>
    //       for the SymboliThread<fn,_,_> part of fn, it's best to leave it to the compiler
    let thread_pool = SimboliThread::<_, 8, 32>::init();

    thread_pool.spawn_task(|| println!("hello world"));

    // the main thread will stop here, waiting for all threads to stop and all tasks to be completed
    thread_pool.join();

    println!("done!")
}
```
