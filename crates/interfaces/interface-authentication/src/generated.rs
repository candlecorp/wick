/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod authenticate {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "authenticate".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      session: payload.consume("session")?,
      username: payload.consume("username")?,
      password: payload.consume("password")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "session")]
    pub session: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "session".to_owned(),
        MessageTransport::success(&inputs.session),
      );

      map.insert(
        "username".to_owned(),
        MessageTransport::success(&inputs.username),
      );

      map.insert(
        "password".to_owned(),
        MessageTransport::success(&inputs.password),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[
    ("session", "string"),
    ("username", "string"),
    ("password", "string"),
  ];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub session: SessionPortSender,
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("session", "string"), ("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct SessionPortSender {
    port: PortChannel,
  }

  impl Default for SessionPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("session"),
      }
    }
  }
  impl PortSender for SessionPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
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
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.session.port, &mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn session(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("session")
        .ok_or_else(|| ComponentError::new("No packets for port 'session' found"))?;
      Ok(PortOutput::new("session".to_owned(), packets))
    }
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod create_user {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "create-user".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      username: payload.consume("username")?,
      password: payload.consume("password")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "username")]
    pub username: String,
    #[serde(rename = "password")]
    pub password: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "user_id".to_owned(),
        MessageTransport::success(&inputs.user_id),
      );

      map.insert(
        "username".to_owned(),
        MessageTransport::success(&inputs.username),
      );

      map.insert(
        "password".to_owned(),
        MessageTransport::success(&inputs.password),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[
    ("user_id", "string"),
    ("username", "string"),
    ("password", "string"),
  ];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod get_id {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "get-id".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      username: payload.consume("username")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "username")]
    pub username: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "username".to_owned(),
        MessageTransport::success(&inputs.username),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("username", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod has_permission {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "has-permission".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      permission: payload.consume("permission")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "permission")]
    pub permission: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "user_id".to_owned(),
        MessageTransport::success(&inputs.user_id),
      );

      map.insert(
        "permission".to_owned(),
        MessageTransport::success(&inputs.permission),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("user_id", "string"), ("permission", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod list_permissions {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-permissions".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "user_id".to_owned(),
        MessageTransport::success(&inputs.user_id),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub permissions: PermissionsPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("permissions", "[string]")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct PermissionsPortSender {
    port: PortChannel,
  }

  impl Default for PermissionsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("permissions"),
      }
    }
  }
  impl PortSender for PermissionsPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.permissions.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn permissions(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("permissions")
        .ok_or_else(|| ComponentError::new("No packets for port 'permissions' found"))?;
      Ok(PortOutput::new("permissions".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod list_users {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "list-users".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      offset: payload.consume("offset")?,
      limit: payload.consume("limit")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "offset")]
    pub offset: u32,
    #[serde(rename = "limit")]
    pub limit: u32,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "offset".to_owned(),
        MessageTransport::success(&inputs.offset),
      );

      map.insert("limit".to_owned(), MessageTransport::success(&inputs.limit));

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("offset", "u32"), ("limit", "u32")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub users: UsersPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("users", "{string:string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UsersPortSender {
    port: PortChannel,
  }

  impl Default for UsersPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("users"),
      }
    }
  }
  impl PortSender for UsersPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.users.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn users(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("users")
        .ok_or_else(|| ComponentError::new("No packets for port 'users' found"))?;
      Ok(PortOutput::new("users".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod remove_user {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "remove-user".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      username: payload.consume("username")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "username")]
    pub username: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "username".to_owned(),
        MessageTransport::success(&inputs.username),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("username", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod update_permissions {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "update-permissions".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      user_id: payload.consume("user_id")?,
      permissions: payload.consume("permissions")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "user_id")]
    pub user_id: String,
    #[serde(rename = "permissions")]
    pub permissions: Vec<String>,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "user_id".to_owned(),
        MessageTransport::success(&inputs.user_id),
      );

      map.insert(
        "permissions".to_owned(),
        MessageTransport::success(&inputs.permissions),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("user_id", "string"), ("permissions", "[string]")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub permissions: PermissionsPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("permissions", "[string]")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct PermissionsPortSender {
    port: PortChannel,
  }

  impl Default for PermissionsPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("permissions"),
      }
    }
  }
  impl PortSender for PermissionsPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.permissions.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn permissions(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("permissions")
        .ok_or_else(|| ComponentError::new("No packets for port 'permissions' found"))?;
      Ok(PortOutput::new("permissions".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
pub mod validate_session {
  use serde::{
    Deserialize,
    Serialize,
  };
  pub use vino_provider::native::prelude::*;

  #[must_use]
  pub fn signature() -> ComponentSignature {
    ComponentSignature {
      name: "validate-session".to_owned(),
      inputs: PortSignature::from_list(inputs_list()),
      outputs: PortSignature::from_list(outputs_list()),
    }
  }

  pub fn populate_inputs(mut payload: TransportMap) -> Result<Inputs, TransportError> {
    Ok(Inputs {
      session: payload.consume("session")?,
    })
  }

  #[derive(Debug, Deserialize, Serialize, Clone)]
  pub struct Inputs {
    #[serde(rename = "session")]
    pub session: String,
  }

  impl From<Inputs> for TransportMap {
    fn from(inputs: Inputs) -> TransportMap {
      let mut map = TransportMap::new();
      map.insert(
        "session".to_owned(),
        MessageTransport::success(&inputs.session),
      );

      map
    }
  }

  static INPUTS_LIST: &[(&str, &str)] = &[("session", "string")];

  #[must_use]
  pub fn inputs_list() -> &'static [(&'static str, &'static str)] {
    INPUTS_LIST
  }

  #[derive(Debug, Default)]
  pub struct OutputPorts {
    pub user_id: UserIdPortSender,
  }

  static OUTPUTS_LIST: &[(&str, &str)] = &[("user_id", "string")];

  #[must_use]
  pub fn outputs_list() -> &'static [(&'static str, &'static str)] {
    OUTPUTS_LIST
  }

  #[derive(Debug)]
  pub struct UserIdPortSender {
    port: PortChannel,
  }

  impl Default for UserIdPortSender {
    fn default() -> Self {
      Self {
        port: PortChannel::new("user_id"),
      }
    }
  }
  impl PortSender for UserIdPortSender {
    fn get_port(&self) -> Result<&PortChannel, ProviderError> {
      if self.port.is_closed() {
        Err(ProviderError::SendChannelClosed)
      } else {
        Ok(&self.port)
      }
    }

    fn get_port_name(&self) -> String {
      self.port.name.clone()
    }
  }

  #[must_use]
  pub fn get_outputs() -> (OutputPorts, TransportStream) {
    let mut outputs = OutputPorts::default();
    let mut ports = vec![&mut outputs.user_id.port];
    let stream = PortChannel::merge_all(&mut ports);
    (outputs, stream)
  }

  #[cfg(feature = "wasm")]
  #[derive(Debug)]
  pub struct Outputs {
    packets: ProviderOutput,
  }

  #[cfg(feature = "wasm")]
  impl Outputs {
    pub fn user_id(&mut self) -> Result<PortOutput> {
      let packets = self
        .packets
        .take("user_id")
        .ok_or_else(|| ComponentError::new("No packets for port 'user_id' found"))?;
      Ok(PortOutput::new("user_id".to_owned(), packets))
    }
  }

  #[cfg(feature = "wasm")]
  impl From<ProviderOutput> for Outputs {
    fn from(packets: ProviderOutput) -> Self {
      Self { packets }
    }
  }
}
