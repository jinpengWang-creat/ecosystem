use std::thread;

use anyhow::Result;
use tokio::fs;

fn main() -> Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()?;

    runtime.block_on(async {
        let a = 10;
        let b = 20;
        println!("{} + {} = {}", a, b, a + b);
    });

    let handler = thread::spawn(|| {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        rt.spawn(async {
            println!("future1");
            let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("{}", content);
        });

        rt.spawn(async {
            println!("future2");
        });

        rt.block_on(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            println!("hello, world");
        });
    });

    handler.join().unwrap();
    Ok(())
}
