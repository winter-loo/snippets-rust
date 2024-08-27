// ideas from https://preshing.com/20120515/memory-reordering-caught-in-the-act/
use std::thread;
use std::sync::mpsc;
use std::sync::atomic::{compiler_fence, fence, Ordering};
use rand::Rng;

static mut X: u64 = 0;
static mut Y: u64 = 0;
static mut R1: u64 = 0;
static mut R2: u64 = 0;

fn worker1(rx: mpsc::Receiver<u8>, tx: mpsc::Sender<u8>) {
    let mut rng = rand::thread_rng();
    loop {
        // Wait for signal from main thread
        rx.recv().unwrap();
        // Add a short, random delay
        while rng.gen_range(0..usize::MAX) % 8 != 0 {}

        unsafe { X = 1; }

        #[cfg(not(feature = "mfence"))]
        compiler_fence(Ordering::SeqCst);
        #[cfg(feature = "mfence")]
        fence(Ordering::SeqCst);

        unsafe { R1 = Y; }

        let _ = tx.send(1);
    }
}

fn worker2(rx: mpsc::Receiver<u8>, tx: mpsc::Sender<u8>) {
    let mut rng = rand::thread_rng();
    loop {
        // Wait for signal from main thread
        rx.recv().unwrap();
        // Add a short, random delay
        while rng.gen_range(0..usize::MAX) % 8 != 0 {}

        unsafe { Y = 1; }

        #[cfg(not(mfence))]
        compiler_fence(Ordering::SeqCst);
        #[cfg(mfence)]
        fence(Ordering::SeqCst);

        unsafe { R2 = X; }

        let _ = tx.send(1);
    }
}

fn main() {
    let (tx1, rx1) =  mpsc::channel::<u8>();
    let (tx2, rx2) =  mpsc::channel::<u8>();
    let (tx3, rx3) =  mpsc::channel::<u8>();

    let tx3_1 = tx3.clone();
    let t1 = thread::spawn(|| {
        worker1(rx1, tx3_1);
    });


    let tx3_2 = tx3.clone();
    let t2 = thread::spawn(|| {
        worker2(rx2, tx3_2);
    });

    let mut detected = 0;
    for i in 0.. {
        unsafe { X = 0; }
        unsafe { Y = 0; }

        let _ = tx1.send(1);
        let _ = tx2.send(1);

        rx3.recv().unwrap();
        rx3.recv().unwrap();

        let r1 = unsafe { R1 };
        let r2 = unsafe { R2 };

        if r1 == 0 && r2 == 0 {
            detected += 1;
            println!("{} reorders detected after {} iterations", detected, i);
        }
    }

    t1.join().unwrap();
    t2.join().unwrap();
}
