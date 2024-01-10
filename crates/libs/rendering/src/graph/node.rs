use bevy_ecs::world::World;
use downcast_rs::{Downcast, impl_downcast};
use avalanche_utils::define_atomic_id;
use crate::prelude::node_slot::SlotInfo;
use crate::prelude::{NodeRunError, RenderingContext};

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
        rendering_context: &mut RenderingContext,
        world: &World,
    ) -> Result<(), NodeRunError>;
}
impl_downcast!(Node);
