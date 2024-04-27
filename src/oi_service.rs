use hyper::body::Body;
use hyper::service::Service;
use hyper::{Request, Response as HyperResponse};
use hyper::body::Incoming;
use http_body_util::Full;
use hyper::body::Bytes;
use std::future::Future as StdFuture;
use crate::typedef::GenericError;
use crate::future_mo::InterceptorFuture;
use crate::load_files::{serve_file, is_file_in_memory};
use crate::service_mo::RequestId;

// type MyAsyncFn = fn(i32) -> Box<dyn Future<Output = i32>>;
type RetFut = Box<dyn std::future::Future<Output = Result<HyperResponse<Full<Bytes>>, GenericError>> + Send>;
type Ret = std::pin::Pin<RetFut>;
type Handler = fn(Request<Incoming>) -> RetFut;

pub struct ServiceWrapper<F, Fut> 
where
    F: Fn(Request<Incoming>) -> Fut,
    Fut: StdFuture<Output = Result<HyperResponse<Full<Bytes>>, GenericError>> + Send,
{
  pub f: F,
}

impl<F, Fut> Service<Request<Incoming>> for ServiceWrapper<F, Fut>
where 
    F: Fn(Request<Incoming>) -> Fut,
    Fut: StdFuture<Output = Result<HyperResponse<Full<Bytes>>, GenericError>> + Send + 'static,

{
    type Response = HyperResponse<Full<Bytes>>;
    type Error = GenericError;
    type Future = InterceptorFuture<Ret>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
      req.extensions_mut().insert(RequestId { id: 42 });
      if is_file_in_memory(req.uri().path()) {
        InterceptorFuture { inner: Box::pin(serve_file(req)) }
      } else {
        InterceptorFuture { inner: Box::pin((self.f)(req)) }
      }
    }
}

