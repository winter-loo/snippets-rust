// modeled as a queue
pub struct List<T> {
    head: Link<T>,
    tail: *mut Node<T>,
}

pub struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            next: std::ptr::null_mut(),
        }
    }
}

type Link<T> = *mut Node<T>;

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: std::ptr::null_mut(),
            tail: std::ptr::null_mut(),
        }
    }

    // push new value at the tail of list
    pub fn push(&mut self, value: T) {
        // Still use Box for heap memory allocation but convert it to raw pointer
        // for later use
        let new_node = Box::into_raw(Box::new(Node::new(value)));
        if self.tail.is_null() {
            assert!(self.head.is_null());
            self.head = new_node;
        } else {
            unsafe {
                (*self.tail).next = new_node;
            }
        }
        self.tail = new_node;
    }

    // pop at the head of the list
    pub fn pop(&mut self) -> Option<T> {
        if !self.head.is_null() {
            // Still use Box to manage heap memory
            let head = unsafe { Box::from_raw(self.head) };
            self.head = head.next;
            Some(head.value)
        } else {
            self.tail = std::ptr::null_mut();
            None
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // 1 -> 2 -> 3

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // 3

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // 3 -> 4 -> 5

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // 5

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // 6 -> 7

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
