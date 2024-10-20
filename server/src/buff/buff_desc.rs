use std::{collections::VecDeque, sync::{Arc, Condvar, Mutex}, thread::{self, ThreadId}};
use crate::{constants::PAGE_SIZE_IN_BYTES, new_page::NewPage, page::Page};
use super::spin_lock::SpinLock;

pub struct BufferDesc {
    mode: LockMode,
    pub is_pinned: bool,
    pub file_name: String,
    pub page: u32,
    s_lock: SpinLock,
    buff: NewPage,
    wait_q: VecDeque<WaitQueueEntry>,
}
#[derive(Clone)]
pub enum LockMode {
    UNLOCKED,
    SHARED(u64), // One or more transactions hold lock, value represents number of transactions,
    EXCLUSIVE(ThreadId) // single transaction holds an exclusive lock on the buffer,
}

struct WaitQueueEntry {
    lock: LockMode,    
    thread_id: ThreadId,
    mut_conv: Arc<(Mutex<()> ,Condvar)>, // needs to be Arc because rust checks when calling wait
}

impl BufferDesc {

    pub fn new(file_name: &str, page: u32, buff: [u8; PAGE_SIZE_IN_BYTES as usize]) -> Self {
        BufferDesc {
            is_pinned: true,
            mode: LockMode::UNLOCKED,
            file_name: file_name.to_string(),
            page,
            s_lock: SpinLock::new(),
            buff: NewPage::new(buff),
            wait_q: VecDeque::new(), // maybe use list, or at lest make size number of threads?
        }
    }

    pub fn get_buff(&self) -> &NewPage {
        return &self.buff
    }

    pub fn write_buff(&mut self, new_buff: NewPage) -> Result<(), String> {
        if let LockMode::EXCLUSIVE(thread_id_holding_lock) = self.mode {
            if thread_id_holding_lock == thread::current().id() {
                self.buff = new_buff;
            } else {
               return Err("Writing failed, tried to do operation from thread which does not hold the lock".into())
            }
        } else {
            return  Err("Unexpected operation, lock isn't exclusive".into())
        }
        Ok(())
    }

    pub fn lock_shared(&mut self) -> Result<(), String> {
        self.s_lock.lock();
        match &self.mode {
            LockMode::UNLOCKED => {  self.mode = LockMode::SHARED(1); },
            LockMode::SHARED(count) => { self.mode = LockMode::SHARED(count + 1); },
            LockMode::EXCLUSIVE(_) => { 
                let mut_conv = Arc::new((Mutex::new(()), Condvar::new()));
                while let LockMode::EXCLUSIVE(_) = self.mode {
                    self.s_lock.lock();
                    let entry = WaitQueueEntry {
                        lock: LockMode::SHARED(1),
                        thread_id: thread::current().id(),
                        mut_conv: mut_conv.clone(),
                    };
                    self.wait_q.push_back(entry);
                    self.s_lock.unlock();
                    mut_conv.1.wait(mut_conv.0.lock().unwrap()).unwrap();
                }
                self.s_lock.lock();
                match &self.mode {
                    LockMode::UNLOCKED => { self.mode = LockMode::SHARED(1);},
                    LockMode::SHARED(count) => { self.mode = LockMode::SHARED(count + 1); },
                    _ => { return Err("Unexpected lock mode of buff after thread wake up".into()); },
                }
            } 
        };
        self.is_pinned = false;
        self.s_lock.unlock();
        Ok(())
    }

    pub fn unlock_shared(&mut self) {
        self.s_lock.lock();
        if let LockMode::SHARED(count) = self.mode {
            if count == 1 {
                self.mode = LockMode::UNLOCKED;
                self.notify_waiting_threads_change_in_lock();
            } else {
                self.mode = LockMode::SHARED(count - 1);
            }
        }
        self.s_lock.unlock();
    }

    pub fn lock_exclusive(&mut self) -> Result<(), String> {
        self.s_lock.lock();
        match self.mode {
            LockMode::UNLOCKED => { self.mode = LockMode::EXCLUSIVE( thread::current().id()); },
            _ => {
                let mut_conv = Arc::new((Mutex::new(()), Condvar::new()));
                while let LockMode::EXCLUSIVE(_) | LockMode::SHARED(_) = self.mode {
                    self.s_lock.lock();
                    let entry = WaitQueueEntry {
                        lock: LockMode::EXCLUSIVE(thread::current().id()),
                        thread_id: thread::current().id(),
                        mut_conv: mut_conv.clone(),
                    };
                    self.wait_q.push_back(entry);
                    self.s_lock.unlock();
                    mut_conv.1.wait(mut_conv.0.lock().unwrap()).unwrap();
                }
                self.s_lock.lock();
                match self.mode {
                    LockMode::UNLOCKED => { self.mode = LockMode::SHARED(1);},
                    _ => { return Err("Unexpected lock mode of buff after thread wake up".into()); },
                }
            } 
        };
        self.is_pinned = false;
        self.s_lock.unlock();
        Ok(())
    }

    pub fn unlock_exclusive(&mut self) -> Result<(),String> {
        self.s_lock.unlock();
        if let LockMode::EXCLUSIVE(_) = self.mode {
                self.mode = LockMode::UNLOCKED;
                self.notify_waiting_threads_change_in_lock();
        } else {
            self.s_lock.unlock();
            return Err("Cant unlock exclusive, lock is not that type".into())
        }
        self.s_lock.unlock();
        Ok(())
    }

    pub fn get_lock_type(&self) -> LockMode {
        return self.mode.clone();     
    }

    fn notify_waiting_threads_change_in_lock(&mut self) {
        if self.wait_q.is_empty() {
            return;
        }
        let last_index_to_notify: usize = if let LockMode::EXCLUSIVE(_) = self.wait_q.front().unwrap().lock {
                0
            } else {
                let mut last_index_to_notify = 0;
                let number_of_waiting_threads = self.wait_q.len();
                for (index, entry) in self.wait_q.iter_mut().enumerate() {
                    if let LockMode::EXCLUSIVE(_) = entry.lock {
                        last_index_to_notify = index - 1;
                    } else {
                        if index == number_of_waiting_threads - 1 {
                            last_index_to_notify = index
                        }
                    }
                }
                last_index_to_notify
        };
        let threads_to_notify: Vec<WaitQueueEntry> = self.wait_q.drain(0..last_index_to_notify).collect(); 
        for entry in threads_to_notify {
            entry.mut_conv.0.lock();
            entry.mut_conv.1.notify_one();
        }   
    }
}