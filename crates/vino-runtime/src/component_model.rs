use actix::Recipient;

use crate::components::{
  Inputs,
  Outputs,
};
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct ComponentModel {
  pub id: String,
  pub name: String,
  pub inputs: Inputs,
  pub outputs: Outputs,
  pub addr: Recipient<Invocation>,
}
