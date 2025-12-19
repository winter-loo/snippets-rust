//
// False sharing happens when two threads on different cores write to different
// variables that just happen to live on the same cache line (nowadays typically 128 bytes)
//
// See crossbeam-utils/src/cache_padded.rs and tokio/src/runtime/task/core.rs
//
use std::sync::atomic::AtomicU64;
use std::time::Instant;
use std::{sync::Arc, thread};

struct Block {
    d1: AtomicU64,
}

#[repr(align(64))]
struct HighPerformanceBlock {
    d1: AtomicU64,
}

fn slow_worker(blks: Arc<[Block]>, index: usize) {
    for _ in 0..100_000_000 {
        blks[index]
            .d1
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

fn fast_worker(blks: Arc<[HighPerformanceBlock]>, index: usize) {
    for _ in 0..100_000_000 {
        blks[index]
            .d1
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

fn schedule_slow_worker() {
    let blks = [
        Block {
            d1: AtomicU64::new(0),
        },
        Block {
            d1: AtomicU64::new(1),
        },
    ];
    let blks = Arc::new(blks);

    let blks1 = blks.clone();
    let blks2 = blks.clone();

    let h1 = thread::spawn(move || slow_worker(blks1, 0));
    let h2 = thread::spawn(move || slow_worker(blks2, 1));

    h1.join().unwrap();
    h2.join().unwrap();
}

fn schedule_fast_worker() {
    let blks = [
        HighPerformanceBlock {
            d1: AtomicU64::new(0),
        },
        HighPerformanceBlock {
            d1: AtomicU64::new(1),
        },
    ];
    let blks = Arc::new(blks);

    let blks1 = blks.clone();
    let blks2 = blks.clone();

    let h1 = thread::spawn(move || fast_worker(blks1, 0));
    let h2 = thread::spawn(move || fast_worker(blks2, 1));

    h1.join().unwrap();
    h2.join().unwrap();
}

fn main() {
    let now = Instant::now();
    schedule_slow_worker();
    println!("slow_worker: {:?}", now.elapsed());

    let now = Instant::now();
    schedule_fast_worker();
    println!("fast_worker: {:?}", now.elapsed());
}
