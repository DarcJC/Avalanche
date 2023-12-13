use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

pub struct IdGenerator32 {
    current_id: AtomicU32,
}

impl IdGenerator32 {
    pub fn new() -> Self {
        Self {
            current_id: AtomicU32::new(1),
        }
    }

    pub fn next_id(&self) -> u32 {
        self.current_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn none() -> u32 {
        0
    }
}

pub struct IdGenerator64 {
    current_id: AtomicU64,
}

impl crate::IdGenerator64 {
    pub fn new() -> Self {
        Self {
            current_id: AtomicU64::new(1),
        }
    }

    pub fn next_id(&self) -> u64 {
        self.current_id.fetch_add(1, Ordering::SeqCst)
    }

    pub fn none() -> u32 {
        0
    }
}
