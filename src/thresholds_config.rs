use std::thread;
use std::time::Duration;

// threshold is based an ordered sequence of sleeps identifed by count (cnt)
// e.g. 0...1...2...3.......4.......5.......6

const SAMPLING_RATE_STAGE1: Duration = Duration::from_micros(100);
const SAMPLING_RATE_STAGE2: Duration = Duration::from_millis(10);
const SAMPLING_RATE_STAGE3: Duration = Duration::from_millis(100);

pub fn sleep_backoff(cnt: u64) {
    if cnt < 100 {
        thread::sleep(SAMPLING_RATE_STAGE1);
        return;
    } else if cnt < 500 {
        thread::sleep(SAMPLING_RATE_STAGE2);
        return;
    } else {
        thread::sleep(SAMPLING_RATE_STAGE3);
    }
}


pub fn inspect_lock(cnt: u64) -> bool {
    (20..25).contains(&cnt) || (500..).contains(&cnt)
}
