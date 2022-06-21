/// Utility type for a Pin<Box<Future<T>>>
pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
