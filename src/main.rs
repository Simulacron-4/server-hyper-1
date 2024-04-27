mod service_mo;
mod future_mo;
mod support;
mod load_files;
mod typedef;
mod box_pack;
mod oi_service;

use crate::service_mo::service_fn;

use std::net::SocketAddr;
use crate::support::TokioIo;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::{Request, Response};
use tokio::net::TcpListener;
use log::info;

use crate::service_mo::RequestId;
use crate::load_files::serve_file;
use crate::typedef::GenericError;
use crate::oi_service::ServiceWrapper;

async fn hello(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, GenericError> {
    let req_id = req.extensions().get::<RequestId>().unwrap();
    //serve_file(req).await
    let result = format!("Hello, World! req_id: {}", req_id.id);
    Ok(Response::new(Full::new(Bytes::from(result))))
    //Ok(Response::new(Full::new(Bytes::from_static(b"Hello, World!"))))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    simple_logger::init().unwrap();

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
                .serve_connection(io, ServiceWrapper {})
                .await
            {
                println!("Error serving connection: {:?}", err);
            }
        });
    }
}

