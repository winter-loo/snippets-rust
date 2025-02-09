#[test]
fn next_back() {
    use std::collections::BTreeMap;

    let mut m = BTreeMap::new();

    m.insert(1, 1);
    m.insert(4, 2);
    m.insert(7, 3);
    m.insert(10, 4);

    let a = m.range(..=5).next_back();
    println!("{a:#?}");
}

#[test]
fn coalesce() {
    use itertools::Itertools;

    let v = vec![1, 1, 2, 3];

    let v: Vec<_> = v
        .iter()
        .map(|x| (*x, *x))
        .coalesce(|mut prev, cur| {
            if prev.0 == cur.0 {
                prev.1 += cur.1;
                Ok(prev)
            } else {
                Err((prev, cur))
            }
        })
        .collect();

    println!("{v:#?}");
}

#[test]
fn weak_pointer() {
    use std::sync::Mutex;
    use std::sync::{Arc, Weak};

    #[derive(Debug)]
    struct Node {
        value: i32,
        // Weak or Arc is immutable data type, in code below,
        // I need mutate `prev` and `next`, so an interior mutable
        // data type is required, hence the Mutex data type.
        prev: Option<Weak<Mutex<Node>>>,
        next: Option<Arc<Mutex<Node>>>,
    }

    let mut n1 = Node {
        value: 1,
        prev: None,
        next: None,
    };

    let n2 = Node {
        value: 2,
        prev: None,
        next: None,
    };

    let n2 = Arc::new(Mutex::new(n2));
    n1.next = Some(n2.clone());

    // if not use Mutex, then code below is wrong:
    // ```
    // let n1 = Arc::new(n1);
    // n2.prev = Some(Arc::downgrade(&n1))
    // ```

    let n1 = Arc::new(Mutex::new(n1));
    {
        let mut guard = n2.lock().unwrap();
        guard.prev = Some(Arc::downgrade(&n1));
    }

    {
        let guard = n1.lock().unwrap();
        let next_guard = guard.next.as_ref().unwrap().lock().unwrap();
        println!("{} -> {}", guard.value, next_guard.value);
    }
}

#[test]
fn new_cyclic() {
    use std::sync::{Arc, Weak};

    #[derive(Debug)]
    struct Node {
        value: i32,
        prev: Option<Weak<Node>>,
        next: Option<Arc<Node>>,
    }

    // 1) Arc::new_cyclic allows to construct this doubly linked list in one step
    // 2) You must use Arc::new_cyclic if you want to create a cyclic structure
    //    where a Node references itself or another node that eventually points
    //    back to it.
    let n = Arc::new_cyclic(
        // Arc::new_cyclic provides a Weak pointer before Arc fully constructed
        |weak_n1| Node {
            value: 1,
            prev: None,
            next: Some(Arc::new(Node {
                value: 2,
                prev: Some(weak_n1.clone()),
                next: None,
            })),
        },
    );
    println!("{} -> {}", n.value, n.next.as_ref().unwrap().value);

    let itself = n.next.as_ref().unwrap().prev.as_ref().unwrap();
    let itself = itself.upgrade().unwrap();
    println!(
        "{} -> {}",
        itself.value,
        itself.next.as_ref().unwrap().value
    );
}

fn main() {}
