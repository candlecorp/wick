/**********************************************
***** This file is generated, do not edit *****
***********************************************/

#[cfg(all(feature = "native", not(feature = "wasm")))]
pub use vino_provider::native::prelude::*;
#[cfg(all(feature = "wasm", not(feature = "native")))]
pub use vino_provider::wasm::prelude::*;

pub mod __multi__;
pub mod delete; // delete
pub mod exists; // exists
pub mod key_get; // key-get
pub mod key_set; // key-set
pub mod list_add; // list-add
pub mod list_range; // list-range
pub mod list_remove; // list-remove
pub mod set_add; // set-add
pub mod set_contains; // set-contains
pub mod set_get; // set-get
pub mod set_remove; // set-remove
pub mod set_scan; // set-scan

#[derive(Debug)]
pub(crate) struct Dispatcher {}
#[async_trait]
impl Dispatch for Dispatcher {
  type Context = crate::Context;
  async fn dispatch(
    op: &str,
    context: Self::Context,
    data: TransportMap,
  ) -> Result<TransportStream, Box<NativeComponentError>> {
    let result = match op {
      "delete" => {
        self::generated::delete::Component::default()
          .execute(context, data)
          .await
      }
      "exists" => {
        self::generated::exists::Component::default()
          .execute(context, data)
          .await
      }
      "key-get" => {
        self::generated::key_get::Component::default()
          .execute(context, data)
          .await
      }
      "key-set" => {
        self::generated::key_set::Component::default()
          .execute(context, data)
          .await
      }
      "list-add" => {
        self::generated::list_add::Component::default()
          .execute(context, data)
          .await
      }
      "list-range" => {
        self::generated::list_range::Component::default()
          .execute(context, data)
          .await
      }
      "list-remove" => {
        self::generated::list_remove::Component::default()
          .execute(context, data)
          .await
      }
      "set-add" => {
        self::generated::set_add::Component::default()
          .execute(context, data)
          .await
      }
      "set-contains" => {
        self::generated::set_contains::Component::default()
          .execute(context, data)
          .await
      }
      "set-get" => {
        self::generated::set_get::Component::default()
          .execute(context, data)
          .await
      }
      "set-remove" => {
        self::generated::set_remove::Component::default()
          .execute(context, data)
          .await
      }
      "set-scan" => {
        self::generated::set_scan::Component::default()
          .execute(context, data)
          .await
      }
      "__multi__" => {
        self::generated::__multi__::Component::default()
          .execute(context, data)
          .await
      }
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub fn get_signature() -> ProviderSignature {
  let mut components = std::collections::HashMap::new();

  components.insert("delete", vino_interface_keyvalue::delete::signature());
  components.insert("exists", vino_interface_keyvalue::exists::signature());
  components.insert("key-get", vino_interface_keyvalue::key_get::signature());
  components.insert("key-set", vino_interface_keyvalue::key_set::signature());
  components.insert("list-add", vino_interface_keyvalue::list_add::signature());
  components.insert("list-range", vino_interface_keyvalue::list_range::signature());
  components.insert("list-remove", vino_interface_keyvalue::list_remove::signature());
  components.insert("set-add", vino_interface_keyvalue::set_add::signature());
  components.insert("set-contains", vino_interface_keyvalue::set_contains::signature());
  components.insert("set-get", vino_interface_keyvalue::set_get::signature());
  components.insert("set-remove", vino_interface_keyvalue::set_remove::signature());
  components.insert("set-scan", vino_interface_keyvalue::set_scan::signature());

  ProviderSignature {
    name: Some("vino-interface-keyvalue".to_owned()),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
  }
}

pub mod types {
  // no additional types
}

pub mod generated {

  // start namespace
  // Component name : delete
  pub mod delete {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::delete::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "delete".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::delete::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : exists
  pub mod exists {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::exists::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "exists".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::exists::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : key-get
  pub mod key_get {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::key_get::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "key-get".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::key_get::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : key-set
  pub mod key_set {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::key_set::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "key-set".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::key_set::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : list-add
  pub mod list_add {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::list_add::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "list-add".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::list_add::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : list-range
  pub mod list_range {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::list_range::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "list-range".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::list_range::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : list-remove
  pub mod list_remove {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::list_remove::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "list-remove".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::list_remove::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : set-add
  pub mod set_add {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::set_add::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "set-add".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::set_add::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : set-contains
  pub mod set_contains {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::set_contains::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "set-contains".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::set_contains::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : set-get
  pub mod set_get {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::set_get::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "set-get".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::set_get::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : set-remove
  pub mod set_remove {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::set_remove::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "set-remove".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::set_remove::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }
  // Component name : set-scan
  pub mod set_scan {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::set_scan::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    pub fn signature() -> ComponentSignature {
      ComponentSignature {
        name: "set-scan".to_owned(),
        inputs: inputs_list().into(),
        outputs: outputs_list().into(),
      }
    }

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::set_scan::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }
  }

  pub mod __multi__ {
    #![allow(unused)]
    use async_trait::async_trait;
    use vino_interface_keyvalue::__multi__::*;
    #[cfg(all(feature = "native", not(feature = "wasm")))]
    pub use vino_provider::native::prelude::*;
    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub use vino_provider::wasm::prelude::*;

    #[derive(Default, Copy, Clone, Debug)]
    pub struct Component {}

    #[async_trait]
    impl NativeComponent for Component {
      type Context = crate::Context;
      async fn execute(
        &self,
        context: Self::Context,
        data: TransportMap,
      ) -> Result<TransportStream, Box<NativeComponentError>> {
        let inputs = populate_inputs(data).map_err(|e| NativeComponentError::new(e.to_string()))?;
        let (outputs, stream) = get_outputs();
        let result = tokio::spawn(crate::components::__multi__::job(inputs, outputs, context))
          .await
          .map_err(|e| Box::new(NativeComponentError::new(format!("Component error: {}", e))))?;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(e.to_string()))),
        }
      }
    }

    pub fn populate_inputs(mut payload: TransportMap) -> Result<Vec<ComponentInputs>, TransportError> {
      payload.consume::<Vec<ComponentInputs>>("inputs")
    }
  }
}
