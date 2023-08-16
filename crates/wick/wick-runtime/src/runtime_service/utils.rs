use flow_graph_interpreter::{HandlerMap, NamespaceHandler};
use wick_config::config::ComponentDefinition;
use wick_config::Resolver;

use super::error::ConstraintFailure;
use super::ChildInit;
use crate::components::{init_hlc_component, init_manifest_component, init_wasm_component};
use crate::dev::prelude::*;
use crate::runtime::RuntimeConstraint;

pub(super) fn assert_constraints(
  constraints: &[RuntimeConstraint],
  components: &HandlerMap,
) -> Result<(), EngineError> {
  for constraint in constraints {
    #[allow(irrefutable_let_patterns)]
    if let RuntimeConstraint::Operation { entity, signature } = constraint {
      let handler = components
        .get(entity.component_id())
        .ok_or_else(|| EngineError::InvalidConstraint(ConstraintFailure::ComponentNotFound(entity.clone())))?;
      let sig = handler.component().signature();
      let op = sig.get_operation(entity.operation_id()).ok_or_else(|| {
        EngineError::InvalidConstraint(ConstraintFailure::OperationNotFound(
          entity.clone(),
          sig.operations.iter().map(|o| o.name().to_owned()).collect(),
        ))
      })?;
      for field in &signature.inputs {
        op.inputs
          .iter()
          .find(|sig_field| sig_field.name == field.name)
          .ok_or_else(|| {
            EngineError::InvalidConstraint(ConstraintFailure::InputNotFound(entity.clone(), field.name.clone()))
          })?;
      }
      for field in &signature.outputs {
        op.outputs
          .iter()
          .find(|sig_field| sig_field.name == field.name)
          .ok_or_else(|| {
            EngineError::InvalidConstraint(ConstraintFailure::OutputNotFound(entity.clone(), field.name.clone()))
          })?;
      }
    }
  }
  Ok(())
}

pub(crate) async fn instantiate_import(
  binding: &config::ImportBinding,
  opts: ChildInit,
  resolver: Box<Resolver>,
) -> Result<Option<NamespaceHandler>, EngineError> {
  opts
    .span
    .in_scope(|| debug!(id = binding.id(), ?opts, "instantiating import"));
  let id = binding.id().to_owned();
  match binding.kind() {
    config::ImportDefinition::Component(c) => instantiate_imported_component(id, c, opts, resolver).await,
    config::ImportDefinition::Types(_) => Ok(None),
  }
}

pub(crate) async fn instantiate_imported_component(
  id: String,
  kind: &ComponentDefinition,
  opts: ChildInit,
  resolver: Box<Resolver>,
) -> Result<Option<NamespaceHandler>, EngineError> {
  match kind {
    #[allow(deprecated)]
    config::ComponentDefinition::Wasm(def) => Ok(Some(
      init_wasm_component(def.reference(), id, opts, None, None, Default::default()).await?,
    )),
    config::ComponentDefinition::Manifest(def) => Ok(Some(init_manifest_component(def, id, opts).await?)),
    config::ComponentDefinition::Reference(_) => unreachable!(),
    config::ComponentDefinition::GrpcUrl(_) => todo!(), // CollectionKind::GrpcUrl(v) => initialize_grpc_collection(v, namespace).await,
    config::ComponentDefinition::HighLevelComponent(hlc) => {
      init_hlc_component(id, opts.root_config.clone(), None, hlc.clone(), resolver)
        .await
        .map(Some)
    }
    config::ComponentDefinition::Native(_) => Ok(None),
  }
}

#[cfg(test)]
mod test {
  // You can find many of the engine tests in the integration tests

  use std::sync::Arc;

  use anyhow::Result;
  use flow_component::Component;
  use wick_interface_types::{component, operation, ComponentSignature};
  use wick_packet::{Entity, Invocation, PacketStream, RuntimeConfig};

  use super::*;

  struct TestComponent {
    signature: ComponentSignature,
  }

  impl TestComponent {
    fn new() -> Self {
      Self {
        signature: component! {
          name: "test",
          version: Some("0.0.1"),
          operations: {
            "testop" => {
              inputs: {
                "in" => "object",
              },
              outputs: {
                "out" => "object",
              },
            },
          }
        },
      }
    }
  }

  impl Component for TestComponent {
    fn handle(
      &self,
      _invocation: Invocation,
      _data: Option<RuntimeConfig>,
      _callback: Arc<RuntimeCallback>,
    ) -> flow_component::BoxFuture<std::result::Result<PacketStream, flow_component::ComponentError>> {
      todo!()
    }

    fn signature(&self) -> &ComponentSignature {
      &self.signature
    }
  }

  #[test]
  fn test_constraints() -> Result<()> {
    let mut components = HandlerMap::default();

    components.add(NamespaceHandler::new("test", Box::new(TestComponent::new())))?;

    let constraints = vec![RuntimeConstraint::Operation {
      entity: Entity::operation("test", "testop"),
      signature: operation!(
        "testop" => {
          inputs: {
            "in" => "object",
          },
          outputs: {
            "out" => "object",
          },
        }
      ),
    }];

    assert_constraints(&constraints, &components)?;

    let constraints = vec![RuntimeConstraint::Operation {
      entity: Entity::operation("test", "testop"),
      signature: operation!(
        "testop" => {
          inputs: {
            "otherin" => "object",
          },
          outputs: {
            "otherout" => "object",
          },
        }
      ),
    }];

    let result = assert_constraints(&constraints, &components);

    assert!(result.is_err());

    Ok(())
  }
}
