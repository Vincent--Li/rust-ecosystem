// tokio asnyc task send message to worker for expensive blocking task
use std::{any, thread, time::Duration};

use anyhow::Result;
use tokio::sync::mpsc;



#[tokio::main] 
async fn main() -> Result<()> {
  let (tx, mut rx ) = mpsc::channel(32);
  
  let handler = worker(rx);
  
  tokio::spawn(async move {
    let mut i = 0;
    loop {
      i += 1;
      println!("send task {}", i);
      tx.send(format!("task {i}")).await?;
    }
    #[allow(unreachable_code)]
    Ok::<(), anyhow::Error>(())
  });

  handler.join().unwrap();
  
  Ok(())
}


fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
  // tokio 适合做io密集型的事情; 计算密集型的任务, 还是托管给thread来完成
  thread::spawn(move || {
    while let Some(s) = rx.blocking_recv() {
      let ret = expensive_block_task(&s);
      println!("result {}", ret);
    }
  })
}

fn expensive_block_task(name: &str) -> String {
  thread::sleep(Duration::from_millis(800));
  blake3::hash(name.as_bytes()).to_string()
}
