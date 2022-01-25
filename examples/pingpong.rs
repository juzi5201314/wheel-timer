use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{accept_async, connect_async, MaybeTlsStream, WebSocketStream};

use wheel_timer2::{Behave, MultiWheel};

const PRECISION: Duration = Duration::from_millis(100);

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut wheel = MultiWheel::new(10, 10, PRECISION);

    let add_handle = wheel.add_handle();

    let addr = server().await;

    for (id, dur) in [
        Duration::from_secs(1),
        Duration::from_secs(3),
        Duration::from_secs(6),
    ]
    .iter()
    .enumerate()
    {
        let ws_stream = Arc::new(Mutex::new(client(addr).await));

        add_handle
            .add_with_ctx(
                move |ws_stream: &mut Arc<Mutex<WebSocketStream<MaybeTlsStream<TcpStream>>>>| {
                    let ws_stream = ws_stream.clone();
                    tokio::spawn(async move {
                        let mut ws_stream = ws_stream.lock().await;
                        ws_stream
                            .send(Message::Ping(id.to_be_bytes().to_vec()))
                            .await
                            .unwrap();
                        assert!(ws_stream.next().await.unwrap().unwrap().is_pong());
                        println!("pong: {}", id);
                    });
                    Behave::Repeat
                },
                *dur,
                ws_stream,
            )
            .unwrap();
    }

    let mut i = tokio::time::interval(PRECISION);
    loop {
        i.tick().await;
        wheel.tick();
    }
}

async fn server() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(accept_connection(listener));
    addr
}

async fn accept_connection(listener: TcpListener) {
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_connection(stream));
    }
}

async fn handle_connection(stream: TcpStream) {
    let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

    while let Some(msg) = ws_stream.next().await {
        let msg = msg.unwrap();
        if let Message::Ping(b) = msg {
            let id = usize::from_be_bytes((&*b).try_into().unwrap());
            println!("ping: {}", id);
            ws_stream.send(Message::Pong(b)).await.unwrap();
        }
    }
}

async fn client(addr: SocketAddr) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
    connect_async(format!("ws://{}", addr)).await.unwrap().0
}
