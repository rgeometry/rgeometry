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
) -> Result<(), Box<dyn Error + Send + Sync>> {
  println!("Incoming TCP connection from: {}", addr);

  let ws_stream = tokio_tungstenite::accept_async(raw_stream)
    .await
    .expect("Error during the websocket handshake occurred");
  println!("WebSocket connection established: {}", addr);

  let (mut outgoing, mut incoming) = ws_stream.split();

  while let Some(msg) = incoming.next().await {
    let msg = msg?;
    if msg.is_close() {
      break;
    }
    let msg = msg.to_text().unwrap();
    let code = match msg.strip_prefix("gist:") {
      None => String::from(msg),
      Some(gist) => {
        reqwest::get(format!("https://gist.github.com/{}/raw", gist))
          .await?
          .text()
          .await?
      }
    };
    match compiler.run(code).await {
      Ok(path) => {
        outgoing
          .send(Message::text(format!("success\n{}", path)))
          .await?
      }
      Err(fail) => {
        outgoing
          .send(Message::text(format!("error\n{}", fail)))
          .await?
      }
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
