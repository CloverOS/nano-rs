//! Framework services and utilities.
//!
//! Use the task helpers inside a request to propagate its tracing context:
//!
//! ```no_run
//! use nano_rs::task;
//!
//! // Call from within a Tokio runtime, for example in an Axum handler.
//! let async_task = task::spawn(async { 42 });
//! let blocking_task = task::spawn_blocking(|| 42);
//! ```

#[allow(unused_imports)]
pub use nano_rs_core::*;
#[allow(unused_imports)]
pub use nano_rs_extra::*;
#[allow(unused_imports)]
pub use nano_rs_macro::*;
// #[allow(unused_imports)]
// pub use nano_rs_rpc::*;
#[allow(unused_imports)]
pub use nano_rs_build::*;
#[allow(unused_imports)]
pub use nano_rs_proc_macro_route::*;
