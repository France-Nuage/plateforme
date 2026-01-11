//! Long-running operation queue and dispatch system.
//!
//! Provides an asynchronous operation queue backed by PostgreSQL for dispatching
//! tasks like relationship writes. Operations are queued via `Operation::dispatch()`
//! and processed by the operation-worker service.

mod operation;

pub use operation::*;
