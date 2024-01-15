use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use bevy_ecs::world::World;
use downcast_rs::{Downcast, impl_downcast};
use avalanche_utils::define_atomic_id;
use crate::prelude::node_slot::{SlotInfo, SlotInfos};
use crate::prelude::{NodeRunError, RenderGraphContext, RenderGraphError, RenderingContext};
use crate::prelude::edge::EdgeInfo;

define_atomic_id!(NodeId);

pub trait Node: Downcast + Send + Sync + 'static {
    /// Describing the input slots of this node.
    ///
    /// Render graph promise these resources are available will [`Node::run`] been invoked.
    fn input(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    /// Describing the output slots of this node.
    fn output(&self) -> Vec<SlotInfo> {
        Vec::new()
    }

    /// Updating internal node state using current render [`World`] prior to the [`Node::run`] function;
    fn update(&mut self, _world: &mut World) {}

    /// Run a pass.
    ///
    /// A **Pass** issues draw calls, updates output slots and
    /// optionally queues up subgraphs for execution.
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        rendering_context: &mut RenderingContext,
        world: &World,
    ) -> Result<(), NodeRunError>;
}
impl_downcast!(Node);

/// A [`NodeLabel`] is used to reference a [`NodeState`] by either its name or [`NodeId`]
/// inside the [`RenderGraph`](super::RenderGraph).
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NodeLabel {
    Id(NodeId),
    Name(Cow<'static, str>),
}

impl From<&NodeLabel> for NodeLabel {
    fn from(value: &NodeLabel) -> Self {
        value.clone()
    }
}

impl From<String> for NodeLabel {
    fn from(value: String) -> Self {
        NodeLabel::Name(value.into())
    }
}

impl From<&'static str> for NodeLabel {
    fn from(value: &'static str) -> Self {
        NodeLabel::Name(value.into())
    }
}

impl From<NodeId> for NodeLabel {
    fn from(value: NodeId) -> Self {
        NodeLabel::Id(value)
    }
}

/// The internal representation of a [`Node`], with all data required
/// by the [`RenderGraph`](super::RenderGraph).
///
/// The `input_slots` and `output_slots` are provided by the `node`.
pub struct NodeState {
    pub id: NodeId,
    pub name: Option<Cow<'static, str>>,
    /// The name of the type that implements [`Node`].
    pub type_name: &'static str,
    pub node: Box<dyn Node>,
    pub input_slots: SlotInfos,
    pub output_slots: SlotInfos,
    pub edges: EdgeInfo,
}

impl Debug for NodeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "[{:?}] {:?} ({:?})", self.type_name, self.id, self.name)
    }
}

impl NodeState {
    /// Create a empty [`NodeState`] using the slots from `node: T`.
    pub fn new<T>(id: NodeId, node: T) -> Self
        where
            T: Node,
    {
        NodeState {
            id,
            name: None,
            input_slots: node.input().into(),
            output_slots: node.output().into(),
            node: Box::new(node),
            type_name: std::any::type_name::<T>(),
            edges: EdgeInfo {
                id,
                input_edges: Vec::new(),
                output_edges: Vec::new(),
            }
        }
    }

    pub fn node<T>(&self) -> Result<&T, RenderGraphError>
        where
            T: Node,
    {
        self.node
            .downcast_ref::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    pub fn node_mut<T>(&mut self) -> Result<&mut T, RenderGraphError>
        where
            T: Node,
    {
        self.node
            .downcast_mut::<T>()
            .ok_or(RenderGraphError::WrongNodeType)
    }

    pub fn validate_input_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.input_slots.len() {
            self.edges.get_input_slot_edge(i)?;
        }

        Ok(())
    }

    pub fn validate_output_slots(&self) -> Result<(), RenderGraphError> {
        for i in 0..self.output_slots.len() {
            self.edges.get_output_slot_edge(i)?;
        }

        Ok(())
    }
}

/// A [`Node`] without any inputs, outputs and subgraphs, which does nothing when run.
/// Used (as a label) to bundle multiple dependencies into one inside
/// the [`RenderGraph`](super::RenderGraph).
#[derive(Default)]
pub struct EmptyNode;

impl Node for EmptyNode {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        _render_context: &mut RenderingContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        Ok(())
    }
}
