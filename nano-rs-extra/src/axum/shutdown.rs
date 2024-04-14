use tokio::signal;

#[cfg(target_family = "unix")]
pub async fn shutdown_signal() {
    let mut term = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("failed to register signal handler");
    let mut interrupt = signal::unix::signal(signal::unix::SignalKind::interrupt())
        .expect("failed to register signal handler");

    tokio::select! {
        _ = term.recv() => {},
        _ = interrupt.recv() => {},
    };
    eprintln!("Project shutting down gracefully...");
}

#[cfg(target_family = "windows")]
pub async fn shutdown_signal() {
    signal::windows::ctrl_c()
        .expect("failed to register signal handler")
        .recv()
        .await;
    eprintln!("Project shutting down gracefully...");
}
