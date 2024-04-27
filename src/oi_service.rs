use hyper::body::Body;
use hyper::service::Service;
use hyper::{Request, Response as HyperResponse};
use hyper::body::Incoming;
use http_body_util::Full;
use hyper::body::Bytes;
use crate::typedef::GenericError;
use crate::future_mo::InterceptorFuture;
use crate::load_files::serve_file;

// type MyAsyncFn = fn(i32) -> Box<dyn Future<Output = i32>>;
type Ret = std::pin::Pin<Box<dyn std::future::Future<Output = Result<HyperResponse<Full<Bytes>>, GenericError>> + Send>>;

pub struct ServiceWrapper {
}

impl Service<Request<Incoming>> for ServiceWrapper 
{
    type Response = HyperResponse<Full<Bytes>>;
    type Error = GenericError;
    type Future = InterceptorFuture<Ret>;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
      InterceptorFuture { inner: Box::pin(serve_file(req)) }
    }
}

