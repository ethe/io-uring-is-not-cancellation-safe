use std::thread;
use std::time::Duration;

use futures::{pin_mut, FutureExt};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use glommio::net::{TcpListener, TcpStream};
use glommio::timer::sleep;
use glommio::{LocalExecutorBuilder, Placement};

fn main() {
    let ex = LocalExecutorBuilder::new(Placement::Unbound)
        .name("server")
        .make()
        .unwrap();

    ex.run(async move {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let ex = LocalExecutorBuilder::new(Placement::Unbound)
                .name("client")
                .make()
                .unwrap();
            ex.run(async move {
                let mut counter = 0;
                loop {
                    let mut stream = TcpStream::connect(addr).await.unwrap();
                    println!("connected {}", counter);
                    let result = stream.write_all(b"hello world").await;
                    result.unwrap();
                    let result = stream.read_exact(&mut vec![0; 11][..]).await;
                    result.unwrap();
                    sleep(Duration::from_millis(100)).await;
                    println!("completed {}", counter);
                    counter += 1;
                }
            });
        });

        loop {
            let t1 = listener.accept().fuse();
            let t2 = sleep(Duration::from_millis(1)).fuse();
            pin_mut!(t1, t2);

            futures::select! {
                stream = t1 => {
                    let mut stream = stream.unwrap();
                    let mut buf = vec![0; 11];
                    let result = stream.read_exact(&mut buf[..]).await;
                    result.unwrap();
                    let result = stream.write_all(&buf).await;
                    result.unwrap();
                }
                _ = t2 => {
                    continue;
                }
            }
        }
    });
}
