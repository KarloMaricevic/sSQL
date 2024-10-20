use std::thread;

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::ThreadId;

pub struct SpinLock {
    locked: AtomicBool,
    thread_id: Option<ThreadId>,
}

impl SpinLock {
    pub fn new() -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            thread_id: None,
        }
    }

    pub fn lock(&mut self) {
        let current_thread_id = thread::current().id();
        if self.thread_id == Some(current_thread_id) {
            panic!("Re-entrant locking is not allowed");
        }
        while self.locked.swap(true, Ordering::Acquire) {
        }
        self.thread_id = Some(current_thread_id);
    }

    pub fn unlock(&mut self) {
        let current_thread_id = thread::current().id();
        if self.thread_id != Some(current_thread_id) {
            panic!("Attempt to unlock a lock not owned by the current thread");
        }
        self.thread_id = None;
        self.locked.store(false, Ordering::Release);
    }
}