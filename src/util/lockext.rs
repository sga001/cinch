use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait RwLockExt<T> {
    fn upgrade(&self, lock: RwLockReadGuard<T>) -> LockResult<RwLockWriteGuard<T>>;
    fn downgrade(&self, lock: RwLockWriteGuard<T>) -> LockResult<RwLockReadGuard<T>>;
}

// Below only works for a single-writer multiple-reader. Multiple-writer multiple-reader
// will result in deadlock if they try to upgrade.

impl<T> RwLockExt<T> for RwLock<T> {
    fn upgrade(&self, lock: RwLockReadGuard<T>) -> LockResult<RwLockWriteGuard<T>> {
        drop(lock);
        self.write()
    }

    fn downgrade(&self, lock: RwLockWriteGuard<T>) -> LockResult<RwLockReadGuard<T>> {
        drop(lock);
        self.read()
    }
}
