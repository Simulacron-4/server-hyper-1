use std::future::Future;
extern crate alloc;
use core::pin::Pin;

pub type BoxFuture<'a, T> = Pin<alloc::boxed::Box<dyn Future<Output = T> + Send + 'a>>;

impl<T: ?Sized> BoxedFuture for T where T: Future {}

pub trait BoxedFuture: Future {
    fn to_boxed<'a>(self) -> BoxFuture<'a, Self::Output>
    where
        Self: Sized + Send + 'a,
    {
        assert_future::<Self::Output, _>(Box::pin(self))
    }
}


pub fn assert_future<T, F>(future: F) -> F
where
    F: Future<Output = T>,
{
    future
}
