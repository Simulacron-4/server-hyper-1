use pin_project_lite::pin_project;
use std::future::Future;
use std::task::Poll;
use std::task::Context;
use std::pin::Pin;


pin_project! {
  pub struct InterceptorFuture<F> {
    #[pin]
    pub inner: F,
  }
}

impl<F> Future for InterceptorFuture<F>
where
    F: Future,
{
    type Output = F::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let polled: Poll<_> = this.inner.poll(cx);

        if polled.is_ready() {
            println!("finished processing request");
        }
        polled
    }
}
