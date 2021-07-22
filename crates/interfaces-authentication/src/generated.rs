/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub mod authenticate {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub session: String,
    pub username: String,
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

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "session")]
    pub session: Vec<u8>,
    #[serde(rename = "username")]
    pub username: Vec<u8>,
    #[serde(rename = "password")]
    pub password: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      session: deserialize(
        map
          .get("session")
          .ok_or_else(|| Error::MissingInput("session".to_owned()))?,
      )?,
      username: deserialize(
        map
          .get("username")
          .ok_or_else(|| Error::MissingInput("username".to_owned()))?,
      )?,
      password: deserialize(
        map
          .get("password")
          .ok_or_else(|| Error::MissingInput("password".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub session: SessionSender,
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("session", "string"), ("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct SessionSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for SessionSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("session".into()))),
      }
    }
  }
  impl Sender for SessionSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }
  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.session.port.clone(), outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod create_user {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub user_id: String,
    pub username: String,
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

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "user_id")]
    pub user_id: Vec<u8>,
    #[serde(rename = "username")]
    pub username: Vec<u8>,
    #[serde(rename = "password")]
    pub password: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      user_id: deserialize(
        map
          .get("user_id")
          .ok_or_else(|| Error::MissingInput("user_id".to_owned()))?,
      )?,
      username: deserialize(
        map
          .get("username")
          .ok_or_else(|| Error::MissingInput("username".to_owned()))?,
      )?,
      password: deserialize(
        map
          .get("password")
          .ok_or_else(|| Error::MissingInput("password".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod get_id {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub username: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("username", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "username")]
    pub username: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      username: deserialize(
        map
          .get("username")
          .ok_or_else(|| Error::MissingInput("username".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod has_permission {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub user_id: String,
    pub permission: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string"), ("permission", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "user_id")]
    pub user_id: Vec<u8>,
    #[serde(rename = "permission")]
    pub permission: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      user_id: deserialize(
        map
          .get("user_id")
          .ok_or_else(|| Error::MissingInput("user_id".to_owned()))?,
      )?,
      permission: deserialize(
        map
          .get("permission")
          .ok_or_else(|| Error::MissingInput("permission".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod list_permissions {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub user_id: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "user_id")]
    pub user_id: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      user_id: deserialize(
        map
          .get("user_id")
          .ok_or_else(|| Error::MissingInput("user_id".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub permissions: PermissionsSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("permissions", "[string]")]
  }

  #[derive(Debug)]
  pub struct PermissionsSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for PermissionsSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("permissions".into()))),
      }
    }
  }
  impl Sender for PermissionsSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.permissions.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod list_users {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub offset: i32,
    pub limit: i32,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("offset", "i32"), ("limit", "i32")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "offset")]
    pub offset: Vec<u8>,
    #[serde(rename = "limit")]
    pub limit: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      offset: deserialize(
        map
          .get("offset")
          .ok_or_else(|| Error::MissingInput("offset".to_owned()))?,
      )?,
      limit: deserialize(
        map
          .get("limit")
          .ok_or_else(|| Error::MissingInput("limit".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub users: UsersSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("users", "{string:string")]
  }

  #[derive(Debug)]
  pub struct UsersSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UsersSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("users".into()))),
      }
    }
  }
  impl Sender for UsersSender {
    type PayloadType = HashMap<String, String>;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.users.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod remove_user {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub username: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("username", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "username")]
    pub username: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      username: deserialize(
        map
          .get("username")
          .ok_or_else(|| Error::MissingInput("username".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod update_permissions {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub user_id: String,
    pub permissions: Vec<String>,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string"), ("permissions", "[string]")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "user_id")]
    pub user_id: Vec<u8>,
    #[serde(rename = "permissions")]
    pub permissions: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      user_id: deserialize(
        map
          .get("user_id")
          .ok_or_else(|| Error::MissingInput("user_id".to_owned()))?,
      )?,
      permissions: deserialize(
        map
          .get("permissions")
          .ok_or_else(|| Error::MissingInput("permissions".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub permissions: PermissionsSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("permissions", "[string]")]
  }

  #[derive(Debug)]
  pub struct PermissionsSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for PermissionsSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("permissions".into()))),
      }
    }
  }
  impl Sender for PermissionsSender {
    type PayloadType = Vec<String>;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.permissions.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
pub mod validate_session {

  use std::collections::HashMap;
  use std::sync::{
    Arc,
    Mutex,
  };

  use serde::{
    Deserialize,
    Serialize,
  };
  use vino_codec::messagepack::deserialize;
  use vino_codec::Error;
  pub use vino_rpc::port::Sender;
  use vino_rpc::port::{
    Port,
    PortStream,
  };

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct Inputs {
    pub session: String,
  }

  #[must_use]
  pub fn inputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("session", "string")]
  }

  #[derive(Debug, PartialEq, Deserialize, Serialize, Default, Clone)]
  pub struct InputEncoded {
    #[serde(rename = "session")]
    pub session: Vec<u8>,
  }

  pub fn deserialize_inputs(
    map: &HashMap<String, Vec<u8>>,
  ) -> Result<Inputs, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Inputs {
      session: deserialize(
        map
          .get("session")
          .ok_or_else(|| Error::MissingInput("session".to_owned()))?,
      )?,
    })
  }

  #[derive(Default, Debug)]
  pub struct Outputs {
    pub user_id: UserIdSender,
  }

  #[must_use]
  pub fn outputs_list() -> Vec<(&'static str, &'static str)> {
    vec![("user_id", "string")]
  }

  #[derive(Debug)]
  pub struct UserIdSender {
    port: Arc<Mutex<Port>>,
  }
  impl Default for UserIdSender {
    fn default() -> Self {
      Self {
        port: Arc::new(Mutex::new(Port::new("user_id".into()))),
      }
    }
  }
  impl Sender for UserIdSender {
    type PayloadType = String;

    fn get_port(&self) -> Arc<Mutex<Port>> {
      self.port.clone()
    }
  }

  pub fn get_outputs() -> (Outputs, PortStream) {
    let outputs = Outputs::default();
    let ports = vec![outputs.user_id.port.clone()];
    let stream = PortStream::new(ports);
    (outputs, stream)
  }
}
