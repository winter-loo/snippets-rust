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
        Self { value, next: None }
    }
}

type Link<T> = Option<Box<Node<T>>>;

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: std::ptr::null_mut(),
        }
    }

    // push new value at the tail of list
    pub fn push(&mut self, value: T) {
        let mut new_node = Box::new(Node::new(value));
        // node_pointer value will not change even box is moved!
        let node_pointer = new_node.as_mut() as *mut _;
        if self.tail.is_null() {
            assert!(self.head.is_none());
            self.head = Some(new_node);
        } else {
            unsafe {
                (*self.tail).next = Some(new_node);
            }
        }
        self.tail = node_pointer;
    }

    // pop at the head of the list
    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            self.head = old_head.next;
            if self.head.is_none() {
                self.tail = std::ptr::null_mut();
            }
            old_head.value
        })
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

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
