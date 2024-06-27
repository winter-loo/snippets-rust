use std::sync::Arc;
use tokio::sync::Mutex as TokioMutex;
use tokio::task;

#[tokio::main]
async fn main() {
    // Create a shared string
    let shared_string = Arc::new(TokioMutex::new(String::new()));

    // Clone the Arc for each task
    let task1_shared_string = shared_string.clone();
    let task2_shared_string = shared_string.clone();

    // Define the tasks
    let task1 = task::spawn(async move {
        let mut guard = task1_shared_string.lock().await;
        guard.push_str("Hello, ");
    });

    let task2 = task::spawn(async move {
        let mut guard = task2_shared_string.lock().await;
        guard.push_str("world!");
    });

    // Await completion of both tasks
    task1.await.unwrap();
    task2.await.unwrap();

    // Access the final string
    let final_string = shared_string.lock().await.clone();
    println!("{}", final_string);
}
