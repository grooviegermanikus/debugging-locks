use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::debugging_locks::RwLockWrapped;

pub fn runit() {


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


