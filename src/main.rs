use std::thread;
use std::time::Duration;

use compio::driver::ProactorBuilder;
use compio::io::{AsyncReadExt, AsyncWriteExt};
use compio::net::{TcpListener, TcpStream};
use compio::time;
use futures::{pin_mut, FutureExt};

#[compio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        compio::runtime::RuntimeBuilder::new()
            .with_proactor(ProactorBuilder::new())
            .build()
            .unwrap()
            .block_on(async {
                let mut counter = 0;
                loop {
                    let mut stream = TcpStream::connect(addr).await.unwrap();
                    println!("connected {}", counter);
                    let result = stream.write_all("hello world").await;
                    result.unwrap();
                    let result = stream.read_exact(vec![0; 11]).await;
                    result.unwrap();
                    println!("completed {}", counter);
                    time::sleep(Duration::from_millis(100)).await;
                    counter += 1;
                }
            });
    });

    loop {
        let t1 = listener.accept().fuse();
        let t2 = time::sleep(Duration::from_millis(1)).fuse();
        pin_mut!(t1, t2);
        futures::select! {
            stream = t1 => {
                let (mut stream, _) = stream.unwrap();
                let (_, buf) = stream.read_exact(vec![0; 11]).await.unwrap();
                let result = stream.write_all(buf).await;
                result.unwrap();
            }
            _ = t2 => {
                continue;
            }
        }
    }
}
