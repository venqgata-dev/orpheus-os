use std::time::Duration;
use std::thread;

fn main() {
    println!("Orpheus OS Agent v0.1 starting...");

    loop {
        println!("Orpheus Agent running...");
        thread::sleep(Duration::from_secs(5));
    }
}
