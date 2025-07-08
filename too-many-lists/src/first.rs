use std::mem;

pub struct List {
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
    pub fn new() -> Self {
        List {
            head: ListImpl::Empty,
        }
    }

    pub fn push(&mut self, elem: usize) {
        let new_node = Box::new(Node {
            value: elem,
            next: mem::replace(&mut self.head, ListImpl::Empty),
        });
        self.head = ListImpl::ListImpl(new_node);
    }

    pub fn pop(&mut self) -> Option<usize> {
        match &mut self.head {
            ListImpl::Empty => None,
            ListImpl::ListImpl(node) => {
                let value = Some(node.value);
                self.head = mem::replace(&mut node.next, ListImpl::Empty);
                value
            }
        }
    }

    pub fn pop2(&mut self) -> Option<usize> {
        match mem::replace(&mut self.head, ListImpl::Empty) {
            ListImpl::Empty => None,
            ListImpl::ListImpl(node) => {
                self.head = node.next;
                Some(node.value)
            }
        }
    }
}

impl Default for List {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }
}
