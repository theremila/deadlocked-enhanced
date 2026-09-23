//! utility types and traits built on top of the standard library.

mod bitset;
pub use bitset::{BitSet, DynamicBitSet};

mod channel;
pub use channel::Channel;

mod future;
pub use future::FutureBlocking;

#[cfg(target_os = "linux")]
pub mod id;

pub mod io;

pub mod log;
pub use log::{Level, LoggerOptions, init, log};

#[cfg(target_os = "linux")]
pub mod meta;

#[cfg(target_os = "linux")]
pub mod path;

mod sync;
pub use sync::{Mutex, RwLock};

pub const fn is_debug() -> bool {
    cfg!(debug_assertions)
}

pub const fn is_test() -> bool {
    cfg!(test)
}
