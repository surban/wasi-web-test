use std::time::Duration;

use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
async fn simple_test() {
    println!("simple_test works");
}

#[wasm_bindgen_test]
async fn thread_test() {
    println!("spawning thread");

    let hnd = std::thread::spawn(move || {
        println!("hello from thread");
        std::thread::sleep(Duration::from_secs(5));
        println!("bye from thread");
    });

    println!("waiting 1s");
    std::thread::sleep(Duration::from_secs(1));

    println!("joining thread");
    hnd.join().unwrap();
    println!("joined thread");
}

#[wasm_bindgen_test]
fn panic_test() {
    panic!("test panic");
}


#[wasm_bindgen_test]
fn async_panic_test() {
    panic!("async test panic");
}