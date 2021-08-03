/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod authenticate {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      session: payload.consume("session")?,
      username: payload.consume("username")?,
      password: payload.consume("password")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "session")]
    pub session: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![
      ("session", "string"),
      ("username", "string"),
      ("password", "string"),
    ]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub session: SessionPortSender,
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("session", "string"), ("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct SessionPortSender {
    port: PortChannel,
  }

  impl Default for SessionPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("session".into()),
      }
    }
  }
  impl PortSender for SessionPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }
  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.session.port, &mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod create_user {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      username: payload.consume("username")?,
      password: payload.consume("password")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![
      ("user_id", "string"),
      ("username", "string"),
      ("password", "string"),
    ]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod get_id {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      username: payload.consume("username")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "username")]
    pub username: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("username", "string")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod has_permission {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      permission: payload.consume("permission")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "permission")]
    pub permission: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string"), ("permission", "string")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod list_permissions {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub permissions: PermissionsPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("permissions", "[string]")]
  }

  #[derive(Debug)]
  pub struct PermissionsPortSender {
    port: PortChannel,
  }

  impl Default for PermissionsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("permissions".into()),
      }
    }
  }
  impl PortSender for PermissionsPortSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.permissions.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod list_users {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      offset: payload.consume("offset")?,
      limit: payload.consume("limit")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "offset")]
    pub offset: u32,
    #[serde(rename = "limit")]
    pub limit: u32,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("offset", "u32"), ("limit", "u32")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub users: UsersPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("users", "{string:string")]
  }

  #[derive(Debug)]
  pub struct UsersPortSender {
    port: PortChannel,
  }

  impl Default for UsersPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("users".into()),
      }
    }
  }
  impl PortSender for UsersPortSender {
    type PayloadType = std::collections::HashMap<String, String>;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.users.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod remove_user {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      username: payload.consume("username")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "username")]
    pub username: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("username", "string")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod update_permissions {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      permissions: payload.consume("permissions")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string"), ("permissions", "[string]")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub permissions: PermissionsPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("permissions", "[string]")]
  }

  #[derive(Debug)]
  pub struct PermissionsPortSender {
    port: PortChannel,
  }

  impl Default for PermissionsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("permissions".into()),
      }
    }
  }
  impl PortSender for PermissionsPortSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.permissions.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
pub mod validate_session {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      session: payload.consume("session")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    #[serde(rename = "session")]
    pub session: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("session", "string")]
  }

  #[derive(Debug, Default)]
  pub struct Outputs {
    pub user_id: UserIdPortSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id".into()),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    type PayloadType = String;

    fn get_port(&self) -> PacketSender {
      self.port.channel.clone().unwrap()
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (Outputs, MessageTransportStream) {
    let mut outputs = Outputs::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }
}
