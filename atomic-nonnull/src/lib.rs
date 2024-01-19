use std::{
    cell::UnsafeCell,
    ptr::NonNull,
    sync::atomic::{AtomicPtr, Ordering},
};

pub struct AtomicNonNull<T> {
    data: UnsafeCell<NonNull<T>>,
}

unsafe impl<T> Sync for AtomicNonNull<T> {}

impl<T> AtomicNonNull<T> {
    pub const fn new(ptr: NonNull<T>) -> Self {
        Self {
            data: UnsafeCell::new(ptr),
        }
    }

    fn as_atomic_ptr(&self) -> &AtomicPtr<T> {
        unsafe { AtomicPtr::from_ptr(self.data.get() as *mut *mut T) }
    }

    pub fn load(&self, order: Ordering) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.as_atomic_ptr().load(order)) }
    }

    pub fn store(&self, ptr: NonNull<T>, order: Ordering) {
        self.as_atomic_ptr().store(ptr.as_ptr(), order);
    }

    pub fn swap(&self, ptr: NonNull<T>, order: Ordering) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.as_atomic_ptr().swap(ptr.as_ptr(), order)) }
    }

    pub fn compare_exchange(
        &self,
        current: NonNull<T>,
        new: NonNull<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<NonNull<T>, NonNull<T>> {
        self.as_atomic_ptr()
            .compare_exchange(current.as_ptr(), new.as_ptr(), success, failure)
            .map(|ptr| unsafe { NonNull::new_unchecked(ptr) })
            .map_err(|ptr| unsafe { NonNull::new_unchecked(ptr) })
    }

    pub fn into_inner(self) -> NonNull<T> {
        self.data.into_inner()
    }

    pub fn get_mut(&mut self) -> &mut NonNull<T> {
        self.data.get_mut()
    }
}
