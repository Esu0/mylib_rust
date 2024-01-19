use std::{cell::Cell, marker::PhantomPinned, ops::Deref, pin::Pin, ptr};

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

    pub fn insert_next<'a, 'b>(self: Pin<&'a Self>, node: Pin<&'b Self>) {
        if &*self as *const _ == &*node {
            return;
        }
        let old_next = self.next.replace(&*node);
        if !old_next.is_null() {
            unsafe {
                (*old_next).prev.set(&*node);
            }
        }
        node.prev.set(&*self);
        node.next.set(old_next);
    }

    pub fn insert_prev<'a, 'b>(self: Pin<&'a Self>, node: Pin<&'b Self>) {
        if &*self as *const _ == &*node {
            return;
        }
        let old_prev = self.prev.replace(&*node);
        if !old_prev.is_null() {
            unsafe {
                (*old_prev).next.set(&*node);
            }
        }
        node.next.set(&*self);
        node.prev.set(old_prev);
    }

    pub fn for_each_to_last(self: Pin<&Self>, mut f: impl FnMut(&T)) {
        let mut node = &*self;
        while {
            f(&node.data);
            !node.next.get().is_null()
        } {
            node = unsafe { &*node.next.get() };
        }
    }

    pub fn for_each_to_first(self: Pin<&Self>, mut f: impl FnMut(&T)) {
        let mut node = &*self;
        while {
            f(&node.data);
            !node.prev.get().is_null()
        } {
            node = unsafe { &*node.prev.get() };
        }
    }

    pub fn map_next<U>(self: Pin<&Self>, f: impl FnOnce(&T) -> U) -> Option<U> {
        let next = self.next.get();
        if next.is_null() {
            None
        } else {
            Some(unsafe { f(&(*next).data) })
        }
    }

    pub fn map_prev<U>(self: Pin<&Self>, f: impl FnOnce(&T) -> U) -> Option<U> {
        let prev = self.prev.get();
        if prev.is_null() {
            None
        } else {
            Some(unsafe { f(&(*prev).data) })
        }
    }

    fn cut_by_ref(&self) {
        let next = self.next.replace(ptr::null());
        let prev = self.prev.replace(ptr::null());
        unsafe {
            if !next.is_null() {
                (*next).prev.set(prev);
            }
            if !prev.is_null() {
                (*prev).next.set(next);
            }
        }
    }

    pub fn cut(self: Pin<&Self>) {
        self.cut_by_ref();
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let next = self.next.get();
        let prev = self.prev.get();
        unsafe {
            if !next.is_null() {
                (*next).prev.set(prev);
            }
            if !prev.is_null() {
                (*prev).next.set(next);
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::pin::pin;

    #[test]
    fn pin_test() {
        let a = pin!(Node::new(10));
        {
            let b = pin!(Node::new(20));
            b.as_ref().insert_next(a.as_ref());
            assert_eq!(a.as_ref().map_prev(|x| *x), Some(20));
        }
        let a_ref = a.as_ref();
        assert!(a_ref.map_prev(|x| *x).is_none());
    }
}
