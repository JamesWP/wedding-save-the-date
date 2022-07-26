use tokio::signal::unix::{signal, SignalKind};

pub async fn termination_future() {
    let mut stream = signal(SignalKind::terminate()).unwrap();
    println!("waiting for signal");
    stream.recv().await;
    println!("done waiting for signal");
}