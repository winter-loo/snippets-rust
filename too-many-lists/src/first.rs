use std::mem;

struct List {
    head: ListImpl,
}

struct Node {
    value: usize,
    next: ListImpl,
}

enum ListImpl {
    Empty,
    ListImpl(Box<Node>),
}

impl List {
    fn new() -> Self {
        List {
            head: ListImpl::Empty,
        }
    }

    fn push(&mut self, elem: usize) {
        let new_node = Box::new(Node {
            value: elem,
            next: mem::replace(&mut self.head, ListImpl::Empty),
        });
        self.head = ListImpl::ListImpl(new_node);
    }

    fn pop(&mut self) -> Option<usize> {
        match &mut self.head {
            ListImpl::Empty => None,
            ListImpl::ListImpl(node) => {
                let value = Some(node.value);
                self.head = mem::replace(&mut node.next, ListImpl::Empty);
                value
            }
        }
    }

    fn pop2(&mut self) -> Option<usize> {
        match mem::replace(&mut self.head, ListImpl::Empty) {
            ListImpl::Empty => None,
            ListImpl::ListImpl(node) => {
                self.head = node.next;
                Some(node.value)
            }
        }
    }
}
