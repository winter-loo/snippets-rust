use std::{marker::PhantomData, ptr::NonNull};

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,

    // From https://rust-unofficial.github.io/too-many-lists/sixth-variance.html
    //
    // In this case I don't think we actually need PhantomData, but
    // any time you do use NonNull (or just raw pointers in general), you
    // should always add it to be safe and make it clear to the compiler
    // and others what you think you're doing.
    //
    // PhantomData is a way for us to give the compiler an extra "example"
    // field that conceptually exists in your type but for various reasons
    // (indirection, type erasure, ...) doesn't. In this case we're using
    // NonNull because we're claiming our type behaves "as if" it stored
    // a value T, so we add a PhantomData to make that explicit.
    _compiler_hint: PhantomData<T>,
}


// From https://rust-unofficial.github.io/too-many-lists/sixth-variance.html
//
// NonNull just abuses the fact that `*const T` is covariant and stores
// that instead, casting back and forth between `*mut T` at the API boundary to
// make it "look like" it's storing a `*mut T`. That's the whole trick! That's
// how collections in Rust are covariant!
type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    value: T,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Self {
        Self {
            front: None,
            back: None,
            value,
        }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            front: None,
            back: None,
            len: 0,
            _compiler_hint: PhantomData,
        }
    }

    pub fn push_front(&mut self, value: T) {
        unsafe {
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(value))));
            if let Some(old_front) = self.front {
                (*old_front.as_ptr()).front = Some(new_node);
                (*new_node.as_ptr()).back = Some(old_front);
            } else {
                self.back = Some(new_node);
            }
            self.front = Some(new_node);
            self.len += 1;
        }
    }

    pub fn push_back(&mut self, value: T) {
        unsafe {
            let new_node = NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(value))));
            if let Some(old_back) = self.back {
                (*old_back.as_ptr()).back = Some(new_node);
                (*new_node.as_ptr()).front = Some(old_back);
            } else {
                self.front = Some(new_node);
            }
            self.back = Some(new_node);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.front.map(|old_front| {
            unsafe {
                // take back the ownership
                let mut old_front = Box::from_raw(old_front.as_ptr());
                // return the value
                let result = old_front.value;

                // move the ownership of the back
                self.front = old_front.back;
                // reset the back of the old front
                old_front.back = None;

                // DONE with old_front

                if let Some(new_front) = self.front {
                    (*new_front.as_ptr()).front = None;
                } else {
                    self.back = None;
                }

                self.len -= 1;
                result
            }
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.back.map(|old_back| {
            unsafe {
                // take back the ownership
                let mut old_back = Box::from_raw(old_back.as_ptr());
                // return the value
                let result = old_back.value;

                // move the ownership of the back
                self.back = old_back.front;
                // reset the back of the old front
                old_back.front = None;

                // DONE with old_back

                if let Some(new_back) = self.back {
                    (*new_back.as_ptr()).back = None;
                } else {
                    self.front = None;
                }

                self.len -= 1;
                result
            }
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn front(&self) -> Option<&T> {
        self.front.map(|front| unsafe { &front.as_ref().value })
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.front
            .map(|mut front| unsafe { &mut (front.as_mut().value) })
    }

    pub fn back(&self) -> Option<&T> {
        self.back.map(|back| unsafe { &back.as_ref().value })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.back
            .map(|mut back| unsafe { &mut (back.as_mut().value) })
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            front: self.front,
            back: self.back,
            len: self.len,
            _compiler_hint: PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            front: self.front,
            back: self.back,
            len: self.len,
            _compiler_hint: PhantomData,
        }
    }
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    // remove the following field to see compiler error
    _compiler_hint: PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.front.map(|front| unsafe {
                self.len -= 1;
                self.front = front.as_ref().back;
                &front.as_ref().value
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|back| unsafe {
                self.len -= 1;
                self.back = back.as_ref().front;
                &back.as_ref().value
            })
        } else {
            None
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _compiler_hint: PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // While self.front == self.back is a tempting condition to check here,
        // it won't do the right for yielding the last element! That sort of
        // thing only works for arrays because of "one-past-the-end" pointers.
        if self.len > 0 {
            // We could unwrap front, but this is safer and easier
            self.front.map(|node| unsafe {
                self.len -= 1;
                self.front = (*node.as_ptr()).back;
                &mut (*node.as_ptr()).value
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.back.map(|node| unsafe {
                self.len -= 1;
                self.back = (*node.as_ptr()).front;
                &mut (*node.as_ptr()).value
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T> IntoIterator for LinkedList<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            list: self,
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T> Send for LinkedList<T> {}
unsafe impl<T> Sync for LinkedList<T> {}
unsafe impl<T> Send for Iter<'_, T> {}
unsafe impl<T> Sync for Iter<'_, T> {}
unsafe impl<T> Send for IterMut<'_, T> {}
unsafe impl<T> Sync for IterMut<'_, T> {}

#[allow(dead_code)]
fn assert_properties() {
    fn is_send<T: Send>() {}
    fn is_sync<T: Sync>() {}

    is_send::<LinkedList<i32>>();
    is_sync::<LinkedList<i32>>();

    is_send::<IntoIter<i32>>();
    is_sync::<IntoIter<i32>>();

    is_send::<Iter<i32>>();
    is_sync::<Iter<i32>>();

    is_send::<IterMut<i32>>();
    is_sync::<IterMut<i32>>();

    fn linked_list_covariant<'a, T>(x: LinkedList<&'static T>) -> LinkedList<&'a T> { x }
    fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> { x }
    fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> { x }
}


#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}
