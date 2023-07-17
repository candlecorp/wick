#![allow(unused)]

pub(crate) type AssociatedData = super::operation_settings::OperationSettings;

pub(crate) type Network = flow_graph::Network<AssociatedData>;
pub(crate) type OperationNode = flow_graph::Node<AssociatedData>;
pub(crate) type OperationPort = flow_graph::NodePort;
pub(crate) type Schematic = flow_graph::Schematic<AssociatedData>;
pub(crate) type Node = flow_graph::Node<AssociatedData>;
pub(crate) type Port<'a> = flow_graph::iterators::Port<'a, AssociatedData>;
