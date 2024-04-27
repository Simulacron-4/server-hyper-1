use hyper::service::Service;
use hyper::{Request, Response as HyperResponse};
use hyper::body::Incoming;
use http_body_util::Full;
use hyper::body::Bytes;
use std::future::Future as StdFuture;
use crate::typedef::GenericError;
use crate::future_mo::InterceptorFuture;
use crate::load_files::{serve_file, is_file_in_memory};
use crate::box_pack::BoxedFuture;

// type MyAsyncFn = fn(i32) -> Box<dyn Future<Output = i32>>;
type OIResponse = HyperResponse<Full<Bytes>>;
type OIResult= Result<OIResponse, GenericError>;
type RetFut = Box<dyn StdFuture<Output = OIResult> + Send>;
type Ret = std::pin::Pin<RetFut>;
trait OIFuture: StdFuture<Output = OIResult> + Send {}

pub struct ServiceWrapper<F, Fut> 
where
    F: Fn(Request<Incoming>) -> Fut,
    Fut: StdFuture<Output = OIResult> + Send,
{
  pub f: F,
}

impl<F, Fut> ServiceWrapper<F, Fut>
where
    F: Fn(Request<Incoming>) -> Fut,
    Fut: StdFuture<Output = OIResult> + Send,
{
    pub fn new(f: F) -> Self {
        ServiceWrapper { f }
    }
}

impl<F, Fut> Service<Request<Incoming>> for ServiceWrapper<F, Fut>
where 
    F: Fn(Request<Incoming>) -> Fut,
    Fut: StdFuture<Output = OIResult> + Send + 'static,
{
    type Response = OIResponse;
    type Error = GenericError;
    type Future = InterceptorFuture<Ret>;

    fn call(&self, mut req: Request<Incoming>) -> Self::Future {
      req.extensions_mut().insert(RequestId { id: 42 });
      if is_file_in_memory(req.uri().path()) {
        // this also works: Box::pin(serve_file(req))
        InterceptorFuture { inner: serve_file(req).to_boxed() }
      } else {
        InterceptorFuture { inner: (self.f)(req).to_boxed() }
      }
    }
}

#[derive(Clone)]
pub struct RequestId {
    pub id: u32,
}
