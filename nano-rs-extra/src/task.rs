//! Tokio task helpers that preserve the caller's tracing span and subscriber.
//!
//! Also available as `nano_rs::task`. These helpers capture context when called,
//! so a task can retain its request ID even after the HTTP response is returned.
//! Use them at every task boundary, including nested tasks. They do not create
//! request IDs or spans when called outside a request, and respect the current
//! subscriber's filters. Raw Tokio spawns, streaming response producers, and
//! WebSocket background tasks still require explicit context propagation.

use std::future::Future;

use tokio::task::JoinHandle;
use tracing::instrument::WithSubscriber;
use tracing::{Instrument, Span};

/// Spawns a future with the current tracing span and subscriber.
///
/// Like [`tokio::spawn`], this must be called from a Tokio runtime. Return
/// values, panics, cancellation, and detaching by dropping the handle follow
/// Tokio's semantics. The span is entered only while the future is polled or
/// dropped, so it does not leak across `.await` points to other tasks.
///
/// ```
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), tokio::task::JoinError> {
/// use nano_rs_extra::task;
///
/// let handle = task::spawn(async {
///     tracing::info!("processing background work");
///     task::spawn(async { 42 }).await
/// });
/// assert_eq!(handle.await??, 42);
/// # Ok(())
/// # }
/// ```
pub fn spawn<F>(future: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future.in_current_span().with_current_subscriber())
}

/// Runs a blocking closure with the current tracing span and subscriber.
///
/// This has the same runtime requirements and [`JoinHandle`] behavior as
/// [`tokio::task::spawn_blocking`]. In particular, a blocking task that has
/// started cannot be aborted. The captured context is entered only for the
/// closure's synchronous execution and restored when it returns or unwinds.
///
/// ```
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), tokio::task::JoinError> {
/// use nano_rs_extra::task;
///
/// let handle = task::spawn_blocking(|| {
///     tracing::info!("processing blocking work");
///     42
/// });
/// assert_eq!(handle.await?, 42);
/// # Ok(())
/// # }
/// ```
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let span = Span::current();
    let dispatcher = tracing::dispatcher::get_default(Clone::clone);
    tokio::task::spawn_blocking(move || {
        tracing::dispatcher::with_default(&dispatcher, || span.in_scope(f))
    })
}
