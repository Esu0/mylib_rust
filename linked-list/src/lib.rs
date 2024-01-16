use std::{ptr, cell::Cell, pin::Pin, marker::PhantomPinned};

pub struct Node<T> {
    data: T,
    next: Cell<*const Node<T>>,
    prev: Cell<*const Node<T>>,
    _marker: PhantomPinned,
}

impl<T> Node<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data,
            next: Cell::new(ptr::null()),
            prev: Cell::new(ptr::null()),
            _marker: PhantomPinned,
        }
    }

    pub fn insert_after<'a, 'b>(self: Pin<&'a Self>, node: Pin<&'b Self>) {
        let old_next = self.next.replace(&*node);
        unsafe {
            (*old_next).prev.set(&*node);
        }
        node.prev.set(&*self);
        node.next.set(old_next);
    }

    pub fn get_mut_by_cut(self: Pin<&mut Self>) -> &mut T
    where
        T: Unpin
    {
        let next = self.next.replace(ptr::null());
        let prev = self.prev.replace(ptr::null());
        unsafe {
            if !next.is_null() {
                (*next).prev.set(prev);
            }
            if !prev.is_null() {
                (*prev).next.set(next);
            }
            &mut self.get_unchecked_mut().data
        }
    }
}

#[cfg(test)]
mod tests {
    use std::pin::pin;
    use super::*;


    #[test]
    fn pin_test() {
        let mut a = pin!(Node::new(10));
        {
            let b = a.as_mut();
        }

    }
}