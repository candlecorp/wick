use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use async_trait::async_trait;
use vino_provider::native::prelude::*;
use vino_rpc::error::RpcError;
use vino_rpc::{
  RpcHandler,
  RpcResult,
};

use crate::generated::{
  self,
  Dispatcher,
};
pub(crate) type Context = Arc<Mutex<State>>;

#[derive(Debug, Default)]
pub struct State {
  pub auth: HashMap<String, String>,
  pub user_ids: HashMap<String, String>,
  pub sessions: HashMap<String, String>,
  pub permissions: HashMap<String, Vec<String>>,
}

#[derive(Clone, Debug, Default)]
#[must_use]
pub struct Provider {
  context: Arc<Mutex<State>>,
}

impl Provider {
  pub fn default() -> Self {
    Self {
      context: Arc::new(Mutex::new(State::default())),
    }
  }
}

#[async_trait]
impl RpcHandler for Provider {
  async fn invoke(&self, entity: Entity, payload: TransportMap) -> RpcResult<BoxedTransportStream> {
    let context = self.context.clone();
    debug!("Dispatching to {}", entity.url());
    let component = entity.name();
    let stream = Dispatcher::dispatch(&component, context, payload)
      .await
      .map_err(|e| RpcError::ProviderError(e.to_string()))?;

    Ok(Box::pin(stream))
  }

  async fn get_list(&self) -> RpcResult<Vec<HostedType>> {
    let components = generated::get_all_components();
    Ok(components.into_iter().map(HostedType::Component).collect())
  }
}

#[cfg(test)]
mod tests {
  use anyhow::Result;
  use rand::distributions::Alphanumeric;
  use rand::Rng;
  use tokio_stream::StreamExt;
  use vino_macros::transport_map;
  use vino_provider::native::prelude::*;

  use super::*;

  fn rand_string() -> String {
    rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(10)
      .map(char::from)
      .collect()
  }

  async fn create_user(provider: &Provider, username: &str, password: &str) -> Result<String> {
    let user_id = rand_string();
    let job_payload = transport_map! {
      "user_id"=> user_id.as_str(),
      "username"=> username,
      "password"=> password,
    };

    let outputs = provider
      .invoke(Entity::component_direct("create-user"), job_payload)
      .await?;

    let outputs: Vec<TransportWrapper> = outputs.collect().await;
    let output = &outputs[0];
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let user_id: String = output.payload.clone().try_into()?;

    println!("user_id: {:?}", user_id);
    // assert_eq!(user_id, username);
    Ok(user_id)
  }

  async fn remove_user(provider: &Provider, username: &str) -> Result<String> {
    let job_payload = transport_map! {
      "username"=> username,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("remove-user"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let user_id: String = output.payload.try_into()?;

    println!("user_id: {:?}", user_id);
    // assert_eq!(user_id, username);
    Ok(user_id)
  }

  async fn list_users(
    provider: &Provider,
    offset: u32,
    limit: u32,
  ) -> Result<HashMap<String, String>> {
    let job_payload = transport_map! {
      "offset"=> offset,
      "limit"=> limit,
    };

    let mut outputs = provider
      .invoke(Entity::component_direct("list-users"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let users: HashMap<String, String> = output.payload.try_into()?;

    println!("list of users: {:?}", users);
    // assert_eq!(user_id, username);
    Ok(users)
  }

  async fn authenticate(
    provider: &Provider,
    username: &str,
    password: &str,
    session: &str,
  ) -> Result<(String, String)> {
    let job_payload = transport_map! {
      "password"=> password,
      "username"=> username,
      "session"=> session,
    };
    let outputs = provider
      .invoke(Entity::component_direct("authenticate"), job_payload)
      .await?;

    let mut session = String::new();
    let mut user_id = String::new();
    // let mut messages = vec![];
    // while let Some(a) = outputs.next().await {
    //   trace!("!!! got something: {:?}", a);
    //   messages.push(a);
    // }

    let messages: Vec<_> = outputs.collect().await;
    assert_eq!(messages.len(), 4);
    for next in messages {
      println!("Got output from [{}]: {:?}", next.port, next.payload);
      if next.payload.is_signal() {
        continue;
      }
      if next.port == "session" {
        let decoded: Result<String, _> = next.payload.try_into();
        if let Ok(s) = decoded {
          session = s;
        }
      } else if next.port == "user_id" {
        let decoded: Result<String, _> = next.payload.try_into();
        if let Ok(s) = decoded {
          user_id = s;
        }
      } else {
        panic!("Got output for unexpected port");
      }
    }

    // println!("session: {}, user_id: {}", session, user_id);
    Ok((session, user_id))
  }

  async fn get_id(provider: &Provider, username: &str) -> Result<String> {
    let job_payload = transport_map! {
      "username"=> username,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("get-id"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let user_id: String = output.payload.try_into()?;

    println!("user_id: {:?}", user_id);
    Ok(user_id)
  }

  async fn validate_session(provider: &Provider, session: &str) -> Result<String> {
    let job_payload = transport_map! {
      "session"=> session,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("validate-session"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let user_id: String = output.payload.try_into()?;

    println!("user_id: {:?}", user_id);
    Ok(user_id)
  }

  async fn update_permissions(
    provider: &Provider,
    user_id: &str,
    perms: &[&str],
  ) -> Result<Vec<String>> {
    let job_payload = transport_map! {
      "user_id"=> user_id,
      "permissions"=> perms,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("update-permissions"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    let permissions: Vec<String> = output.payload.try_into()?;
    assert_eq!(permissions, perms);

    Ok(permissions)
  }

  async fn has_permission(
    provider: &Provider,
    user_id: &str,
    perm: &str,
  ) -> Result<MessageTransport> {
    let job_payload = transport_map! {
      "user_id"=> user_id,
      "permission"=> perm,
    };
    let mut outputs = provider
      .invoke(Entity::component_direct("has-permission"), job_payload)
      .await?;

    let output = outputs.next().await.unwrap();
    println!("payload from [{}]: {:?}", output.port, output.payload);
    Ok(output.payload)
  }

  #[test_logger::test(tokio::test)]
  async fn test_create_user() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";
    let uid = create_user(&provider, username, password).await?;
    let uid2 = get_id(&provider, username).await?;
    assert_eq!(uid, uid2);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list_users() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";
    create_user(&provider, username, password).await?;
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_list_and_remove_user() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";
    let users = list_users(&provider, 0, 100).await?;
    assert_eq!(users.len(), 0);
    let uid = create_user(&provider, username, password).await?;
    let users = list_users(&provider, 0, 100).await?;
    assert_eq!(users.len(), 1);
    let uid2 = remove_user(&provider, username).await?;
    assert_eq!(uid, uid2);
    let users = list_users(&provider, 0, 100).await?;
    assert_eq!(users.len(), 0);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_authenticate() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";
    let session_in = "generic_session_id";
    let uid = create_user(&provider, username, password).await?;
    trace!("uid from create_user is {}", uid);
    let (session_out, user_id) = authenticate(&provider, username, password, session_in).await?;
    trace!("user_id from authenticate is {}", user_id);
    trace!("session is {}", session_out);
    assert_eq!(session_out, session_in);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_validate_session() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";
    let session_in = "generic_session_id";

    let uid = create_user(&provider, username, password).await?;
    trace!("uid from create_user is {}", uid);
    let (session_out, uid2) = authenticate(&provider, username, password, session_in).await?;
    trace!("session is {}", session_out);
    let uid3 = validate_session(&provider, &session_out).await?;
    assert_eq!(uid, uid2);
    assert_eq!(uid2, uid3);

    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_update_perms() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";

    let uid = create_user(&provider, username, password).await?;
    let perms_in = ["something", "else"];
    let perms_out = update_permissions(&provider, &uid, &perms_in).await?;
    println!("permissions out: {:?}", perms_out);
    assert_eq!(perms_out, perms_in);
    Ok(())
  }

  #[test_logger::test(tokio::test)]
  async fn test_has_perm() -> Result<()> {
    let provider = Provider::default();
    let username = "user@foo.com";
    let password = "password123";

    let uid = create_user(&provider, username, password).await?;
    let perms_in = ["can_do"];
    let perms_out = update_permissions(&provider, &uid, &perms_in).await?;
    println!("permissions {:?}", perms_out);

    let result = has_permission(&provider, &uid, "can_do").await?;
    println!("{:?}", result);
    let uid_out: String = result.try_into()?;
    assert_eq!(uid_out, uid);
    let result = has_permission(&provider, &uid, "can't_do").await?;
    let expected_err = format!(
      "User ID '{}' does not have permission '{}'",
      uid, "can't_do"
    );
    assert_eq!(result, MessageTransport::exception(expected_err));
    Ok(())
  }
}
