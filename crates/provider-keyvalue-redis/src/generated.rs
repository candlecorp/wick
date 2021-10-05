/**********************************************
***** This file is generated, do not edit *****
***********************************************/

pub(crate) use vino_provider::native::prelude::*;

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
    "key-increment".to_owned(),
    vino_interface_keyvalue::key_increment::signature(),
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
    "set-intersection".to_owned(),
    vino_interface_keyvalue::set_intersection::signature(),
  );
  components.insert(
    "set-remove".to_owned(),
    vino_interface_keyvalue::set_remove::signature(),
  );
  components.insert(
    "set-union".to_owned(),
    vino_interface_keyvalue::set_union::signature(),
  );

  ProviderSignature {
    name: "".to_owned(),
    types: StructMap::todo(),
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
        self::delete::Component::default()
          .execute(context, data)
          .await
      }
      "exists" => {
        self::exists::Component::default()
          .execute(context, data)
          .await
      }
      "key-get" => {
        self::key_get::Component::default()
          .execute(context, data)
          .await
      }
      "key-increment" => {
        self::key_increment::Component::default()
          .execute(context, data)
          .await
      }
      "key-set" => {
        self::key_set::Component::default()
          .execute(context, data)
          .await
      }
      "list-add" => {
        self::list_add::Component::default()
          .execute(context, data)
          .await
      }
      "list-range" => {
        self::list_range::Component::default()
          .execute(context, data)
          .await
      }
      "list-remove" => {
        self::list_remove::Component::default()
          .execute(context, data)
          .await
      }
      "set-add" => {
        self::set_add::Component::default()
          .execute(context, data)
          .await
      }
      "set-contains" => {
        self::set_contains::Component::default()
          .execute(context, data)
          .await
      }
      "set-get" => {
        self::set_get::Component::default()
          .execute(context, data)
          .await
      }
      "set-intersection" => {
        self::set_intersection::Component::default()
          .execute(context, data)
          .await
      }
      "set-remove" => {
        self::set_remove::Component::default()
          .execute(context, data)
          .await
      }
      "set-union" => {
        self::set_union::Component::default()
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

pub(crate) mod delete {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::delete::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::exists::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::key_get::*;
  use vino_provider::native::prelude::*;

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
pub(crate) mod key_increment {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::key_increment::*;
  use vino_provider::native::prelude::*;

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
      let result = crate::components::key_increment::job(inputs, outputs, context).await;
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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::key_set::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::list_add::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::list_range::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::list_remove::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_add::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_contains::*;
  use vino_provider::native::prelude::*;

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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_get::*;
  use vino_provider::native::prelude::*;

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
pub(crate) mod set_intersection {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_intersection::*;
  use vino_provider::native::prelude::*;

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
      let result = crate::components::set_intersection::job(inputs, outputs, context).await;
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
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_remove::*;
  use vino_provider::native::prelude::*;

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
pub(crate) mod set_union {
  #![allow(unused)]
  use std::collections::HashMap;

  use async_trait::async_trait;
  use vino_interface_keyvalue::set_union::*;
  use vino_provider::native::prelude::*;

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
      let result = crate::components::set_union::job(inputs, outputs, context).await;
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
