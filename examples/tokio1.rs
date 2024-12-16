use std::{thread, time::Duration};

use tokio::{fs, runtime::Builder, time::sleep};

fn main() {
    let handle = thread::spawn(|| {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();

		rt.spawn(async {
			println!("Future 1!");
			let content = fs::read_to_string("Cargo.toml").await.unwrap();
            println!("content lent {}", content.len());
		});

        rt.spawn(async {
            println!("Future 2!");
            let ret = expensive_block_task("Future 2");
            println!("result : {}", ret);
        });

        rt.block_on(async {
            sleep(Duration::from_millis(900)).await;
        })
    });

    handle.join().unwrap();
}

fn expensive_block_task(name: &str) -> String {
    thread::sleep(std::time::Duration::from_millis(800));
    blake3::hash(name.as_bytes()).to_string()
}
