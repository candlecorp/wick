/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub(crate) use vino_provider::prelude::*;

pub mod delete;
pub mod exists;
pub mod key_get;
pub mod key_set;
pub mod list_add;
pub mod list_range;
pub mod list_remove;
pub mod set_add;
pub mod set_contains;
pub mod set_get;
pub mod set_remove;
pub mod set_scan;

pub(crate) fn get_signature() -> ProviderSignature {
  use std::collections::HashMap;
  let mut components = HashMap::new();
  components.insert(
    "delete".to_owned(),
    vino_interface_keyvalue::delete::signature(),
  );
  components.insert(
    "exists".to_owned(),
    vino_interface_keyvalue::exists::signature(),
  );
  components.insert(
    "key-get".to_owned(),
    vino_interface_keyvalue::key_get::signature(),
  );
  components.insert(
    "key-set".to_owned(),
    vino_interface_keyvalue::key_set::signature(),
  );
  components.insert(
    "list-add".to_owned(),
    vino_interface_keyvalue::list_add::signature(),
  );
  components.insert(
    "list-range".to_owned(),
    vino_interface_keyvalue::list_range::signature(),
  );
  components.insert(
    "list-remove".to_owned(),
    vino_interface_keyvalue::list_remove::signature(),
  );
  components.insert(
    "set-add".to_owned(),
    vino_interface_keyvalue::set_add::signature(),
  );
  components.insert(
    "set-contains".to_owned(),
    vino_interface_keyvalue::set_contains::signature(),
  );
  components.insert(
    "set-get".to_owned(),
    vino_interface_keyvalue::set_get::signature(),
  );
  components.insert(
    "set-remove".to_owned(),
    vino_interface_keyvalue::set_remove::signature(),
  );
  components.insert(
    "set-scan".to_owned(),
    vino_interface_keyvalue::set_scan::signature(),
  );

  ProviderSignature {
    name: Some("vino-interface-keyvalue".to_owned()),
    types: std::collections::HashMap::from([]).into(),
    components: components.into(),
  }
}

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
      _ => Err(Box::new(NativeComponentError::new(format!(
        "Component not found on this provider: {}",
        op
      )))),
    }?;
    Ok(result)
  }
}

pub mod generated {
  pub(crate) mod delete {
    #![allow(unused)]
    use vino_interface_keyvalue::delete::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::delete::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod exists {
    #![allow(unused)]
    use vino_interface_keyvalue::exists::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::exists::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod key_get {
    #![allow(unused)]
    use vino_interface_keyvalue::key_get::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::key_get::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod key_set {
    #![allow(unused)]
    use vino_interface_keyvalue::key_set::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::key_set::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod list_add {
    #![allow(unused)]
    use vino_interface_keyvalue::list_add::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::list_add::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod list_range {
    #![allow(unused)]
    use vino_interface_keyvalue::list_range::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::list_range::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod list_remove {
    #![allow(unused)]
    use vino_interface_keyvalue::list_remove::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::list_remove::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod set_add {
    #![allow(unused)]
    use vino_interface_keyvalue::set_add::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::set_add::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod set_contains {
    #![allow(unused)]
    use vino_interface_keyvalue::set_contains::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::set_contains::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod set_get {
    #![allow(unused)]
    use vino_interface_keyvalue::set_get::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::set_get::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod set_remove {
    #![allow(unused)]
    use vino_interface_keyvalue::set_remove::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::set_remove::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
  pub(crate) mod set_scan {
    #![allow(unused)]
    use vino_interface_keyvalue::set_scan::*;

    use std::collections::HashMap;

    use async_trait::async_trait;
    use vino_provider::prelude::*;

    #[derive(Default)]
    pub(crate) struct Component {}

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
        let result = crate::components::set_scan::job(inputs, outputs, context).await;
        match result {
          Ok(_) => Ok(stream),
          Err(e) => Err(Box::new(NativeComponentError::new(format!(
            "Job failed: {}",
            e.to_string()
          )))),
        }
      }
    }
  }
}
