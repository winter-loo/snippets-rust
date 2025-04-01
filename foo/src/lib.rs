// this module adds some functionality based on the required implementations
// here like: `LinkedList::pop_back` or `Clone for LinkedList<T>`
// You are free to use anything in it, but it's mainly for the test framework.
mod pre_implemented;

struct Node<T> {
    value: T,
    prev: *mut Node<T>,
    next: *mut Node<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        }
    }
}

pub struct LinkedList<T> {
    head: *mut Node<T>,
    tail: *mut Node<T>,
    len: usize,
}

pub struct Cursor<'a, T> {
    node: Option<&'a mut Node<T>>,
}

pub struct Iter<'a, T> {
    this: &'a LinkedList<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        LinkedList {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
            len: 0,
        }
    }

    // You may be wondering why it's necessary to have is_empty()
    // when it can easily be determined from len().
    // It's good custom to have both because len() can be expensive for some types,
    // whereas is_empty() is almost always cheap.
    // (Also ask yourself whether len() is expensive for LinkedList)
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn len(&self) -> usize {
        self.len
    }

    /// Return a cursor positioned on the front element
    pub fn cursor_front(&mut self) -> Cursor<'_, T> {
        Cursor {
            node: unsafe { self.head.as_mut() },
        }
    }

    /// Return a cursor positioned on the back element
    pub fn cursor_back(&mut self) -> Cursor<'_, T> {
        Cursor {
            node: unsafe { self.tail.as_mut() },
        }
    }

    /// Return an iterator that moves from front to back
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            this: self,
        }
    }
}

// the cursor is expected to act as if it is at the position of an element
// and it also has to work with and be able to insert into an empty list.
impl<T> Cursor<'_, T> {
    /// Take a mutable reference to the current element
    pub fn peek_mut(&mut self) -> Option<&mut T> {
        todo!()
    }

    /// Move one position forward (towards the back) and
    /// return a reference to the new position
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&mut T> {
        todo!()
    }

    /// Move one position backward (towards the front) and
    /// return a reference to the new position
    pub fn prev(&mut self) -> Option<&mut T> {
        todo!()
    }

    /// Remove and return the element at the current position and move the cursor
    /// to the neighboring element that's closest to the back. This can be
    /// either the next or previous position.
    pub fn take(&mut self) -> Option<T> {
        todo!()
    }

    pub fn insert_after(&mut self, element: T) {
        let node = Node {
            value: element,
            prev: std::ptr::null_mut(),
            next: std::ptr::null_mut(),
        };
        self.node
    }

    pub fn insert_before(&mut self, _element: T) {
        todo!()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        todo!()
    }
}
