use std::future::Future;
use tokio::sync::{mpsc, oneshot};

// mpsc::channel::<(Command, oneshot::Sender<u64>)>(100);
// (Sender<T>, Receiver<T>)
#[derive(Debug, Clone)]
pub struct Manager<I, O> {
  sender: mpsc::Sender<(I, oneshot::Sender<O>)>,
}

impl<I, O> Manager<I, O>
where
  I: Send + 'static,
  O: Send + 'static,
{
  pub async fn new<F, X>(handler: F) -> Manager<I, O>
  where
    F: Fn(I) -> X + Sync + Send + 'static,
    X: Future<Output = O> + Send,
  {
    let (cmd_tx, mut cmd_rx) = mpsc::channel::<(I, oneshot::Sender<O>)>(10);
    tokio::spawn(async move {
      while let Some(boxed) = cmd_rx.recv().await {
        let (cmd, response) = boxed;
        let res = handler(cmd).await;
        response.send(res).ok().unwrap()
      }
    });
    Manager { sender: cmd_tx }
  }

  pub async fn query(&self, arg: I) -> O {
    let (resp_tx, resp_rx) = oneshot::channel();

    self.sender.send((arg, resp_tx)).await.ok().unwrap();
    resp_rx.await.unwrap()
  }
}
