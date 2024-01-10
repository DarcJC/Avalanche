use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use once_cell::sync::Lazy;

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

pub static ID_GENERATOR_32_STATIC: Lazy<IdGenerator32> = Lazy::new(IdGenerator32::new);

pub static ID_GENERATOR_64_STATIC: Lazy<IdGenerator64> = Lazy::new(IdGenerator64::new);

#[macro_export]
macro_rules! define_atomic_id {
    ($atomic_id_type:ident) => {
        #[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
        pub struct $atomic_id_type(core::num::NonZeroU32);

        impl $atomic_id_type {
            pub fn new() -> Self {
                use std::sync::atomic::{AtomicU32, Ordering};

                static COUNTER: AtomicU32 = AtomicU32::new(1);

                let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
                Self (
                    core::num::NonZeroU32::new(counter).unwrap_or_else(|| {
                        panic!(
                            "The system ran out of unique `{}`s.",
                            stringify!($atomic_id_type)
                        );
                    })
                )
            }
        }
    };
}
