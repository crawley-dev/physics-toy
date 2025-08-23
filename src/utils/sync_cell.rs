use std::cell::UnsafeCell;
use std::fmt::{Debug, Formatter, Result};

// This is a simple wrapper on UnsafeCell for parallelism. (impl Sync)
// UnsafeCell is an unsafe primitive for interior mutability (bypassing borrow checker)
// UnsafeCell provides no thread safety gurantees, I don't care though so I made this wrapper
pub struct SyncCell<T>(UnsafeCell<T>);
unsafe impl<T: Send> Sync for SyncCell<T> {}
impl<T> SyncCell<T> {
    pub const fn new(val: T) -> Self {
        Self(UnsafeCell::new(val))
    }

    pub fn get(&self) -> &T {
        unsafe { &*self.0.get() }
    }

    pub fn get_mut(&self) -> &mut T {
        unsafe { &mut *self.0.get() }
    }
}

impl<T: Debug> Debug for SyncCell<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let item = self.get();
        f.debug_struct("SyncCell").field("Item", item).finish()
    }
}

impl<T: Clone> Clone for SyncCell<T> {
    fn clone(&self) -> Self {
        Self::new(self.get().clone())
    }
}
