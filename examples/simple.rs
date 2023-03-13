use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use env_logger::Env;
use rust_debugging_locks::debugging_locks::RwLockWrapped;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    runit();
//
}


fn runit() {


    let lock : Arc<RwLockWrapped<HashMap<i32,i32>>> = Arc::new(RwLockWrapped::new(HashMap::new()));

    let l1 = lock.clone();
    thread::spawn(move || {
        let r1 = l1.read().unwrap();
        println!("acquire read lock {} ...", r1.len());
        thread::sleep(Duration::from_millis(2000));
        println!("... release read lock");
    });

    thread::sleep(Duration::from_millis(500));

    {

        // let mut w = lock.write().unwrap();
        let mut _w = lock.write().unwrap();
        // exclusive lock -> wait

    }


}


