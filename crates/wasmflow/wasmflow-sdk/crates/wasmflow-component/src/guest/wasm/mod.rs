pub mod error;
pub mod exports;
pub mod imports;
pub mod runtime;

use error::Error;
use yielding_async_executor::single_threaded as executor;
/// Utility type for a Pin<Box<Future<T>>>
pub type BoxedFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'static>>;
