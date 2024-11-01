use std::thread;
use std::time::Duration;

use monoio::io::{AsyncReadRentExt, AsyncWriteRentExt};
use monoio::net::{TcpListener, TcpStream};
use monoio::time::TimeDriver;
use monoio::{select, time, IoUringDriver};

// #[monoio::main(driver = "legacy", enable_timer = true)]
#[monoio::main(driver = "io_uring", enable_timer = true)]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    thread::spawn(move || {
        monoio::start::<TimeDriver<IoUringDriver>, _>(async move {
            let mut counter = 0;
            loop {
                let mut stream = TcpStream::connect(addr).await.unwrap();
                println!("connected {}", counter);
                let (result, _) = stream.write_all("hello world").await;
                result.unwrap();
                let (result, _) = stream.read_exact(vec![0; 11]).await;
                result.unwrap();
                time::sleep(Duration::from_millis(100)).await;
                println!("completed {}", counter);
                counter += 1;
            }
        });
    });

    loop {
        select! {
            stream = listener.accept() => {
                let (mut stream, _) = stream.unwrap();
                let (result, buf) = stream.read_exact(vec![0; 11]).await;
                result.unwrap();
                let (result, _) = stream.write_all(buf).await;
                result.unwrap();
            }
            _ = time::sleep(Duration::from_millis(1)) => {
                continue;
            }
        }
    }
}
