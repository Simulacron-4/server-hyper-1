mod service_mo;
mod future_mo;
mod support;
mod load_files;
mod typedef;
mod box_pack;
mod oi_service;
mod websocket;

use std::net::SocketAddr;
use crate::support::TokioIo;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use tokio::net::TcpListener;
use log::info;

use crate::oi_service::RequestId;
use crate::typedef::GenericError;
use crate::oi_service::ServiceWrapper;

async fn hello(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
    let req_id = req.extensions().get::<RequestId>().unwrap();
    let result = format!("Hello, World! req_id: {}", req_id.id);
    if hyper_tungstenite::is_upgrade_request(&req) {
      websocket::handle_websocket_request(req).await
    } else {
      Ok(Response::new(Full::new(Bytes::from(result))))
    }
    //Ok(Response::new(Full::new(Bytes::from_static(b"Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    simple_logger::init_with_env().unwrap();

    // next two lines are for tracing
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    tracing::subscriber::set_global_default(subscriber)?;

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // We create a TcpListener and bind it to 127.0.0.1:3000
    let listener = TcpListener::bind(addr).await?;

    info!("Listening on http://{}", addr);

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                //.serve_connection(io, service_fn(hello))
                .serve_connection(io, ServiceWrapper::new(hello))
                .with_upgrades()
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

