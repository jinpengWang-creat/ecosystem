use std::thread;

use anyhow::Result;
use tokio::sync::mpsc; // Add this import statement
#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(32);
    let handler = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            tx.send(format!("task: {}", i)).await?;
            i += 1;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    handler.join().unwrap();
    Ok(())
}

fn worker(mut receiver: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(msg) = receiver.blocking_recv() {
            let ret = expensive_time_op(msg);
            println!("Received: {}", ret);
        }
    })
}

fn expensive_time_op(msg: String) -> String {
    thread::sleep(std::time::Duration::from_secs(1));
    blake3::hash(msg.as_bytes()).to_string()
}
