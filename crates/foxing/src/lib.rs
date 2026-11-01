//! Foxing driver. Pre-alpha: the primitives arrive one crate at a time, and this crate
//! assembles them. For now it re-exports them so `foxing::time`, `foxing::log` and
//! `foxing::store` are stable paths.

pub use foxing_log as log;
pub use foxing_store as store;
pub use foxing_time as time;
