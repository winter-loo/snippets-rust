use std::sync::Arc;
use tokio::sync::Semaphore;

fn main() {
    let sem = Arc::new(Semaphore::new(12));
    {
        let permit = sem.try_acquire_many(3).unwrap();
        assert_eq!(sem.available_permits(), 9);
        permit.forget();
    }

    // Since we forgot the permit, available permits won't go back to its initial value
    // even after the permit is dropped.
    assert_eq!(sem.available_permits(), 9);
}
