// accept ws connection
// read program source
// iff has wasm, return wasm hash code
// else:
// aquire lock
// write program to playground/wasm/lib/user.rs
// compile
// copy wasm file to ~/.cache/rgeometry/[hash].wasm
// release lock
// send response with the hash of the file.

use std::error::Error;
use std::{env, io::Error as IoError, net::SocketAddr, sync::Arc};

use futures_util::{SinkExt, StreamExt};

use tokio::net::{TcpListener, TcpStream};
use tungstenite;
use tungstenite::protocol::Message;

mod manager;
use manager::Manager;

mod compile;
use compile::compile;

type Compiler = Manager<String, Result<String, Box<dyn Error + Send + Sync>>>;

async fn handle_connection(
  compiler: Arc<Compiler>,
  raw_stream: TcpStream,
  addr: SocketAddr,
) -> tungstenite::Result<()> {
  println!("Incoming TCP connection from: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(raw_stream)
    .await
    .expect("Error during the websocket handshake occurred");
  println!("WebSocket connection established: {}", addr);

  let (mut outgoing, mut incoming) = ws_stream.split();

  loop {
    match incoming.next().await {
      Some(msg) => {
        let msg = msg?;
        if msg.is_close() {
          break;
        }
        match compiler.run(String::from(msg.to_text().unwrap())).await {
          Ok(path) => {
            outgoing
              .send(Message::text(format!("Success: {:?}", path)))
              .await?
          }
          Err(fail) => {
            outgoing
              .send(Message::text(format!("Failed!\n{}", fail)))
              .await?
          }
        }
      }
      None => break,
    }
  }
  Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), IoError> {
  let compiler = Arc::new(Manager::new(compile).await);
  let addr = env::args()
    .nth(1)
    .unwrap_or_else(|| "0.0.0.0:20162".to_string());

  // Create the event loop and TCP listener we'll accept connections on.
  let try_socket = TcpListener::bind(&addr).await;
  let listener = try_socket.expect("Failed to bind");
  println!("Listening on: {}", addr);

  // Let's spawn the handling of each connection in a separate task.
  while let Ok((stream, addr)) = listener.accept().await {
    tokio::spawn(handle_connection(compiler.clone(), stream, addr));
  }

  Ok(())
}
