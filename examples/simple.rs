use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use env_logger::Env;
use rust_debugging_locks::debugging_locks::RwLockWrapped;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    runit();
}


fn runit() {


    let lock : Arc<RwLockWrapped<HashMap<i32,i32>>> = Arc::new(RwLockWrapped::new(HashMap::new()));

    let l1 = lock.clone();
    thread::spawn(move || {
        let r1 = l1.read().unwrap();
        println!("acquire read lock {} ...", r1.len());
        thread::sleep(Duration::from_millis(100));
        println!("... release read lock.");
    });
    // wait unit r1 lock is aquired
    thread::sleep(Duration::from_millis(50));


    println!("acquiring writer lock ...");
    let mut _w = lock.write().unwrap();
    println!("... writer lock acquired.");

    let l2 = lock.clone();
    println!("acquiring read2 lock ...");
    let _r2 = l2.read().unwrap();
    thread::sleep(Duration::from_millis(100));

}


