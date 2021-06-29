use crate::components::{
  Inputs,
  Outputs,
};

#[derive(Debug, Clone)]
pub struct ComponentModel {
  pub id: String,
  pub name: String,
  pub inputs: Inputs,
  pub outputs: Outputs,
}
