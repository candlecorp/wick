use crate::dev::prelude::*;

#[derive(Message)]
#[rtype(result = "Result<Recipient<InvocationMessage>,NetworkError>")]
pub(crate) struct GetRecipient {
  pub(crate) entity: Entity,
}

impl Handler<GetRecipient> for NetworkService {
  type Result = Result<Recipient<InvocationMessage>, NetworkError>;

  fn handle(&mut self, msg: GetRecipient, _ctx: &mut Context<Self>) -> Self::Result {
    self.ensure_is_started()?;
    let err = Err(NetworkError::InvalidRecipient(msg.entity.url()));
    let not_found = NetworkError::UnknownProvider(msg.entity.url());
    let result = match &msg.entity {
      Entity::Invalid => err,
      Entity::System(_) => err,
      Entity::Test(_) => err,
      Entity::Client(_) => err,
      Entity::Host(_) => err,
      Entity::Schematic(_) => self.providers.get(SELF_NAMESPACE).ok_or(not_found),
      Entity::Component(ns, _) => self.providers.get(ns).ok_or(not_found),
      Entity::Provider(name) => self.providers.get(name).ok_or(not_found),
      Entity::Reference(_) => err,
    };
    result.map(|channel| channel.recipient.clone())
  }
}
