use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use env_logger::Env;
use rust_debugging_locks::debugging_locks::RwLockWrapped;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    reader_blocks_writer();
    writer_blocks_reader();

}


fn reader_blocks_writer() {
    let lock : Arc<RwLockWrapped<HashMap<i32,i32>>> = Arc::new(RwLockWrapped::new(HashMap::new()));

    let l1 = lock.clone();
    thread::spawn(move || {
        let r1 = l1.read().unwrap();
        println!("acquire read lock {} ...", r1.len());
        thread::sleep(Duration::from_millis(500));
        println!("... release read lock.");
    });
    // wait unit r1 lock is acquired
    thread::sleep(Duration::from_millis(50));

    println!("acquiring writer lock ...");
    let mut _writer_lock = lock.write().unwrap();
    println!("... writer lock acquired");

}


fn writer_blocks_reader() {
    let lock : Arc<RwLockWrapped<HashMap<i32,i32>>> = Arc::new(RwLockWrapped::new(HashMap::new()));

    let l1 = lock.clone();
    thread::spawn(move || {
        let w1 = l1.write().unwrap();
        println!("acquire write lock {} ...", w1.len());
        thread::sleep(Duration::from_millis(2500));
        println!("... release write lock.");
    });
    // wait unit w1 lock is acquired
    thread::sleep(Duration::from_millis(50));

    println!("acquiring read2 lock ...");
    let _reader_lock = lock.read().unwrap();
    println!("... release read2 lock.");

}

