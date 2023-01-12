use rust_server_listener::consts::messages::BAD_REQUEST;
use rust_server_listener::parsers::websocket_server::handshake;
use rust_server_listener::websocket::websocket::WebSocket;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

extern crate base64;
extern crate sha1_smol;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();

        tokio::spawn(async {
            let stream_result = handshake(&mut stream).await;
            if stream_result.is_err() {
                stream.write_all(BAD_REQUEST.as_bytes()).await.unwrap_or(());
                return;
            }

            let mut websocket = WebSocket::new(stream);
            async fn handle_message(msg: Vec<u8>) {
                println!("i received message: {}", std::str::from_utf8(&msg).unwrap());
                ()
            }

            websocket.listen_for_messages(&handle_message).await;

            async fn callback(a: &mut WebSocket) {
                a.send_message_as_text("aaa");
            }
        });
    }
}
