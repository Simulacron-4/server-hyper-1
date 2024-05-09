use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};
use hyper_tungstenite::{tungstenite, HyperWebsocket};
use tungstenite::Message;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

use log::info;

use crate::typedef::GenericError;

pub async fn handle_websocket_request(mut req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
  info!("Initiating websocket upgrade");
  let (response, websocket) = hyper_tungstenite::upgrade(&mut req, None)?;
  tokio::spawn(async move {
    if let Err(err) = serve_websocket(websocket).await {
      info!("Error serving websocket: {:?}", err);
    }
  });
  Ok(response)
}

async fn serve_websocket(websocket: HyperWebsocket) -> Result<(), GenericError> {
  info!("Serving websocket");
  let mut websocket = websocket.await?;
  info!("Websocket connected");
  while let Some(message) = websocket.next().await {
    match message? {
      Message::Text(text) => {
        info!("Received a text message on websocket: {}", text);
        websocket.send(Message::text("Thank you")).await?;
      }
      _ => {
        info!("Received a non-text message on websocket");
      }
    }
  }    
  Ok(())
}
