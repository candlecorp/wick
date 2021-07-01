use std::fmt;
use std::pin::Pin;

use actix::dev::MessageResponse;
use actix::prelude::*;
use tokio::sync::oneshot::Sender as OneshotSender;
enum ActorResponseTypeItem<A, I> {
  Result(I),
  Fut(Pin<Box<dyn ActorFuture<A, Output = I>>>),
}

/// A helper type that extends actix's own ActorResponse
pub(crate) struct ActorResult<A, I> {
  item: ActorResponseTypeItem<A, I>,
}

impl<A, I> fmt::Debug for ActorResult<A, I> {
  fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut fmt = fmt.debug_struct("ActorResult");
    match self.item {
      ActorResponseTypeItem::Result(_) => fmt.field("item", &"Result(_)"),
      ActorResponseTypeItem::Fut(_) => fmt.field("item", &"Fut(_)"),
    }
    .finish()
  }
}

impl<A: Actor, I> ActorResult<A, I> {
  /// Creates a response.
  pub(crate) fn reply(val: I) -> Self {
    Self {
      item: ActorResponseTypeItem::Result(val),
    }
  }

  /// Creates an asynchronous response.
  pub(crate) fn reply_async<T>(fut: T) -> Self
  where
    T: ActorFuture<A, Output = I> + 'static,
  {
    Self {
      item: ActorResponseTypeItem::Fut(Box::pin(fut)),
    }
  }
}

impl<A, M> MessageResponse<A, M> for ActorResult<A, M::Result>
where
  A: Actor,
  M: Message,
  A::Context: AsyncContext<A>,
{
  fn handle(self, ctx: &mut A::Context, tx: Option<OneshotSender<M::Result>>) {
    match self.item {
      ActorResponseTypeItem::Fut(fut) => {
        let fut = fut.map(|res, _, _| tx.send(res));
        ctx.spawn(fut);
      }
      ActorResponseTypeItem::Result(res) => tx.send(res),
    }
  }
}

// Helper trait for send one shot message from Option<Sender> type.
// None and error are ignored.
trait OneshotSend<M> {
  fn send(self, msg: M);
}

impl<M> OneshotSend<M> for Option<OneshotSender<M>> {
  fn send(self, msg: M) {
    if let Some(tx) = self {
      let _ = tx.send(msg);
    }
  }
}
