#![allow(dead_code)]
#![no_main]

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_thread;
use web_sys::WorkerGlobalScope;

use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{mpsc, Arc, LazyLock, Mutex};
use std::time::Duration;

use tokio::time::sleep;

pub struct DropMsg(pub &'static str);

impl Drop for DropMsg {
    fn drop(&mut self) {
        println!("dropping: {}", self.0);
    }
}

thread_local! {
    pub static TLS: Cell<DropMsg> = Cell::new(DropMsg("thread init value"));
    pub static TLS_B: RefCell<DropMsg> = RefCell::new(DropMsg("thread init value B"));
}

fn run() {
    println!("Hello, world!");

    let (tx, rx) = mpsc::sync_channel(1);

    println!("starting thread");
    let thread = std::thread::spawn(|| thread_fn(rx));
    println!("id is {:?}", thread.thread().id());

    tx.send("hello to thread".into()).unwrap();

    // while !thread.is_finished() {
    //     println!("waiting for thread");
    //     std::thread::sleep(Duration::from_secs(10));
    // }

    println!("Bye");
}

fn run2() {
    println!("Hello, world!");

    //test_sql();

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .enable_time()
        .build()
        .unwrap();
    let res = rt.block_on(async_run());

    println!("async_main result: {res}");

    let (tx, rx) = mpsc::sync_channel(1);

    println!("starting thread");
    let thread = std::thread::spawn(|| thread_fn(rx));
    println!("id is {:?}", thread.thread().id());

    println!("Sending message");
    tx.send("hello to thread".into()).unwrap();

    println!("waiting for thread");
    thread.join().unwrap();

    rt.block_on(async {
        println!("Final 10 s wait");
        sleep(Duration::from_secs(10)).await;
        println!("Final wait done");
    });
    println!("Done");
}

fn thread_fn(rx: mpsc::Receiver<String>) {
    println!("Hello from thread");

    let msg = rx.recv().unwrap();
    println!("Thread received: {msg}");
}

async fn async_run() -> u32 {
    println!("async hello");

    tokio::spawn(async_bg());

    sleep(Duration::from_secs(5)).await;

    tokio::task::spawn_blocking(|| {
        println!("hey from blocing task");
        std::thread::sleep(Duration::from_secs(5));
        println!("blocking task done");
    });

    println!("async done");

    123
}

async fn async_bg() {
    for i in 1..30 {
        println!("background {i}");
        sleep(Duration::from_secs(1)).await;
    }
}

#[wasm_bindgen]
pub fn get_time() -> String {
    let now = std::time::Instant::now();
    format!("{now:?}")
}

#[wasm_bindgen]
pub async fn async_sleep(timeout: i32) {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let tx_opt = Mutex::new(Some(tx));

    let window = web_sys::window().unwrap();
    let handler = Closure::wrap(Box::new(move || {
        let mut tx_opt = tx_opt.lock().unwrap();
        if let Some(tx) = tx_opt.take() {
            let _ = tx.send(());
        }
    }) as Box<dyn Fn()>);

    window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            handler.as_ref().unchecked_ref(),
            timeout,
        )
        .unwrap();

    rx.await.unwrap();
}

#[wasm_bindgen]
pub fn sync_double(x: u8) -> u8 {
    println!("sync double called with {x}");
    eprintln!("sync double called with {x}");
    x * 2
}

#[wasm_bindgen]
pub fn incall_js_type(data: &MyJsType) {
    outcall_js_type(data);
}

#[wasm_bindgen]
extern "C" {
    #[derive(Clone)]
    pub type MyJsType;

    fn outcall_js_type(data: &MyJsType);
}

#[wasm_bindgen]
pub struct ContainingType {
    value: MyJsType,
}

#[wasm_bindgen]
impl ContainingType {
    #[wasm_bindgen(constructor)]
    pub fn new(value: MyJsType) -> Self {
        Self { value }
    }

    pub fn get_value(&self) -> MyJsType {
        self.value.clone()
    }
}

#[wasm_bindgen]
pub fn various_tests() {
    println!("various tests");

    let args: Vec<_> = std::env::args().collect();
    println!("args: {args:?}");

    std::env::set_var("RUST_BACKTRACE", "1");
    let vars: Vec<_> = std::env::vars().collect();
    println!("env vars: {vars:?}");

    let dir = std::env::current_dir();
    println!("current dir: {dir:?}");

    let exe = std::env::current_exe();
    println!("current exe: {exe:?}");

    //std::env::current_exe().expect("testing panic");

    println!("writing /hello.txt");
    std::fs::write("/hello.txt", "Hello!!").unwrap();

    println!("reading /");
    let dir = std::fs::read_dir("/").unwrap();
    for entry in dir {
        let entry = entry.unwrap();
        println!("dir entry: {entry:?}");
    }

    println!("reading /hello.txt");
    let txt = std::fs::read_to_string("/hello.txt").unwrap();
    println!("contents: {txt}");
}

#[wasm_bindgen]
pub fn thread_test() {
    println!(
        "main available parallelism: {:?}",
        std::thread::available_parallelism()
    );

    TLS.set(DropMsg("main 1"));
    TLS.set(DropMsg("main 2"));

    println!("spawning thread");
    let hnd = std::thread::spawn(|| {
        println!("hello from thread");

        println!(
            "thread available parallelism: {:?}",
            std::thread::available_parallelism()
        );

        println!("setting TLS values");
        println!("TLS B is {}", TLS_B.with(|b| b.borrow().0));
        TLS.set(DropMsg("thread set value 1"));
        TLS.set(DropMsg("thread set value 2"));
        TLS_B.set(DropMsg("thread set value 2 B"));

        let dir = std::env::current_dir();
        println!("thread current dir: {dir:?}");

        FOO.set(MyType(2));
        FOO.set(MyType(3));

        println!("Thread sleeping for 1 seconds");
        std::thread::sleep(Duration::from_secs(1));

        let cnt = AtomicU32::new(0);
        HANDLER.set(Some(Closure::wrap(Box::new(move || {
            let cnt = cnt.fetch_add(1, Ordering::AcqRel);
            println!("Callback {cnt} from thread sleep");

            if cnt < 2 {
                HANDLER.with(|handler| {
                    let handler = handler.borrow();
                    let handler = handler.as_ref().unwrap();

                    let global = js_sys::global();
                    let scope = global.dyn_ref::<WorkerGlobalScope>().unwrap();
                    scope
                        .set_timeout_with_callback_and_timeout_and_arguments_0(
                            handler.as_ref().unchecked_ref(),
                            3000,
                        )
                        .unwrap();
                });
            } else {
                println!("Releasing thread");
                unsafe { wasm_bindgen_futures::thread::thread_release() };
                println!("Releasing thread requested");
            }
        }) as Box<dyn Fn()>)));

        println!("Registering setTimeout callback");
        HANDLER.with(|handler| {
            let handler = handler.borrow();
            let handler = handler.as_ref().unwrap();

            let global = js_sys::global();
            let scope = global.dyn_ref::<WorkerGlobalScope>().unwrap();
            scope
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    handler.as_ref().unchecked_ref(),
                    3000,
                )
                .unwrap();
        });

        println!("setTimeout callback registered");

        println!("thread_hold");
        unsafe { wasm_bindgen_futures::thread::thread_hold() };
        println!("thread_hold done");

        println!("Thread main exiting");
        123
    });

    println!("thread spawned with id={:?}", hnd.thread().id());

    let hnd2 = std::thread::spawn(move || {
        println!("waiting for thread join");
        let res = hnd.join();
        println!("thread joined: {res:?}");
    });

    wasm_bindgen_futures::spawn_local(async move {
        while !hnd2.is_finished() {
            println!("thread2 is alive");
            async_sleep(1000).await;
        }
        println!("thread2 finished");

        println!("TLS B: {}", TLS_B.with(|msg| msg.borrow().0));
    });
}

#[wasm_bindgen]
pub fn mutex_test() {
    let mutex = Arc::new(Mutex::new(0));
    let mutex1 = mutex.clone();
    std::thread::spawn(move || {
        println!("thread 4 starting");
        println!("thread 4 locking mutex");
        let mut _guard = mutex1.lock().unwrap();
        println!("thread 4 locked mutex");
        std::thread::sleep(Duration::from_secs(10));
        *_guard = 1;
        println!("thread 4 released mutex")
    });

    let mutex2 = mutex.clone();
    std::thread::spawn(move || {
        println!("thread 5 starting");
        std::thread::sleep(Duration::from_secs(1));
        println!("thread 5 locking mutex");
        let _guard = mutex2.lock().unwrap();
        println!("thread 5 locked mutex with value: {}", *_guard);
        std::thread::sleep(Duration::from_secs(2));
        println!("thread 5 released mutex")
    });
}

static MAIN_MUTEX: LazyLock<Arc<Mutex<u32>>> = LazyLock::new(|| Arc::new(Mutex::new(0)));

#[wasm_bindgen]
pub fn main_mutex_block() {
    let mutex = MAIN_MUTEX.clone();
    std::thread::spawn(move || {
        let mut guard = mutex.lock().unwrap();
        println!("thread acquired mutex");
        std::thread::sleep(Duration::from_secs(10));
        *guard = 11;
        println!("thread released mutex");
    });
}

#[wasm_bindgen]
pub fn main_mutex_read() {
    let mutex = MAIN_MUTEX.clone();
    println!("main acquiring mutex");
    let guard = mutex.lock().unwrap();
    println!("main acquired mutex with value={}", *guard);
}

pub struct MyType(u32);

impl Drop for MyType {
    fn drop(&mut self) {
        println!("dropping: {}", self.0);
    }
}

thread_local! {
    pub static FOO: Cell<MyType> = Cell::new(MyType(1));
    static HANDLER: RefCell<Option<Closure<dyn Fn()>>> = RefCell::new(None);
}

#[wasm_bindgen]
pub async fn async_thread_test() {
    println!(
        "main available parallelism: {:?}",
        std::thread::available_parallelism()
    );

    println!("Spawning async thread");
    let t = spawn_thread(|| async move {
        println!("hello from async thread");

        println!(
            "thread available parallelism: {:?}",
            std::thread::available_parallelism()
        );

        println!("setting TLS values");
        println!("TLS B is {}", TLS_B.with(|b| b.borrow().0));
        TLS.set(DropMsg("thread set value 1"));
        TLS.set(DropMsg("thread set value 2"));
        TLS_B.set(DropMsg("thread set value 2 B"));

        let dir = std::env::current_dir();
        println!("thread current dir: {dir:?}");

        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

        let mut cnt = 0;
        let handler = Closure::new(move || {
            cnt += 1;
            println!("Callback {cnt} from thread sleep");
            tx.send(cnt).unwrap();
        });

        let global = js_sys::global();
        let scope = global.dyn_ref::<WorkerGlobalScope>().unwrap();
        scope
            .set_interval_with_callback_and_timeout_and_arguments(
                handler.as_ref().unchecked_ref(),
                1000,
                &Array::new(),
            )
            .unwrap();

        while let Some(cnt) = rx.recv().await {
            println!("Received {cnt}");
            if cnt == 3 {
                println!("Quitting");
                break;
            }
        }

        println!("Async thread exiting");
        123
    });

    println!("joining async thread");
    let res = t.await;
    println!("async thread result: {res:?}");
}
