use std::error::Error as StdError;
use std::fmt;
use std::future::Future;
use std::marker::PhantomData;

use hyper::body::Body;
use hyper::service::Service;
use hyper::{Request, Response};

use crate::future_mo::LoggingFuture;

/// Create a `Service` from a function.
///
/// # Example
///
/// ```
/// use bytes::Bytes;
/// use hyper::{body, Request, Response, Version};
/// use http_body_util::Full;
/// use hyper::service::service_fn;
///
/// let service = service_fn(|req: Request<body::Incoming>| async move {
///     if req.version() == Version::HTTP_11 {
///         Ok(Response::new(Full::<Bytes>::from("Hello World")))
///     } else {
///         // Note: it's usually better to return a Response
///         // with an appropriate StatusCode instead of an Err.
///         Err("not HTTP/1.1, abort connection")
///     }
/// });
/// ```
pub fn service_fn<F, R, S>(f: F) -> ServiceFn<F, R>
where
    F: Fn(Request<R>) -> S,
    S: Future,
{
    ServiceFn {
        f,
        _req: PhantomData,
    }
}

/// Service returned by [`service_fn`]
pub struct ServiceFn<F, R> {
    f: F,
    _req: PhantomData<fn(R)>,
}

impl<F, ReqBody, Ret, ResBody, E> Service<Request<ReqBody>> for ServiceFn<F, ReqBody>
where
    F: Fn(Request<ReqBody>) -> Ret,
    ReqBody: Body,
    Ret: Future<Output = Result<Response<ResBody>, E>>,
    E: Into<Box<dyn StdError + Send + Sync>>,
    ResBody: Body,
{
    type Response = crate::Response<ResBody>;
    type Error = E;
    type Future = LoggingFuture<Ret>;

    // do extra work layer here
    fn call(&self, mut req: Request<ReqBody>) -> Self::Future {
        req.extensions_mut().insert(RequestId { id: 42 });
        // check if accepted
        //

        let fut = (self.f)(req);
        let logging_future = LoggingFuture { inner: fut };
//        fut.and_then(|result| async move {println!("Ready!"); Ok(())});
        /*
        fut.then(|result| {
            match result {
                Ok(res) => {
                    let mut res = crate::Response::from(res);
                    //res.headers_mut().insert("x-service", "service_fn");
                    Ok(result)
                }
                Err(e) => Err(result),
            }
        });
        */
        logging_future
    }
}

#[derive(Clone)]
pub struct RequestId {
    pub id: u32,
}

impl<F, R> fmt::Debug for ServiceFn<F, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("impl Service").finish()
    }
}

impl<F, R> Clone for ServiceFn<F, R>
where
    F: Clone,
{
    fn clone(&self) -> Self {
        ServiceFn {
            f: self.f.clone(),
            _req: PhantomData,
        }
    }
}

impl<F, R> Copy for ServiceFn<F, R> where F: Copy {}

