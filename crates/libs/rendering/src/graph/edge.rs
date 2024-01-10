use crate::prelude::node::NodeId;
use crate::prelude::RenderGraphError;

/// Provided a coarse grained [`Edge::NodeEdge`] and
/// a fine grained [`Edge::SlotEdge`] dependency between nodes in a graph.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Edge {
    SlotEdge {
        input_node: NodeId,
        input_index: usize,
        output_node: NodeId,
        output_index: usize,
    },
    NodeEdge {
        input_node: NodeId,
        output_node: NodeId,
    },
}

impl Edge {
    pub fn get_input_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { input_node, .. } | Edge::NodeEdge { input_node, .. } => *input_node
        }
    }

    pub fn get_output_node(&self) -> NodeId {
        match self {
            Edge::SlotEdge { output_node, .. } | Edge::NodeEdge { output_node, .. } => *output_node
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum EdgeExistence {
    Exists,
    DoesNotExist,
}

pub struct EdgeInfo {
    pub(crate) id: NodeId,
    pub(crate) input_edges: Vec<Edge>,
    pub(crate) output_edges: Vec<Edge>,
}

impl EdgeInfo {
    #[inline]
    pub fn input_edges(&self) -> &[Edge] {
        &self.input_edges
    }

    #[inline]
    pub fn output_edges(&self) -> &[Edge] {
        &self.output_edges
    }

    #[inline]
    pub fn id(&self) -> NodeId {
        self.id
    }

    pub fn has_input_edge(&self, edge: &Edge) -> bool {
        self.input_edges.contains(edge)
    }

    pub fn has_output_edge(&self, edge: &Edge) -> bool {
        self.output_edges.contains(edge)
    }

    /// Add an edge to `input_edges` if not exist.
    pub(crate) fn add_input_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_input_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.input_edges.push(edge);
        Ok(())
    }

    /// Remove an edge from `input_edges` it exist.
    pub(crate) fn remove_input_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if let Some(index) = self.input_edges.iter().position(|e| *e == edge) {
            self.input_edges.swap_remove(index);
            Ok(())
        } else {
            Err(RenderGraphError::EdgeDoesNotExist(edge))
        }
    }

    /// Add an edge to `output_edges` if not exist.
    pub(crate) fn add_output_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if self.has_output_edge(&edge) {
            return Err(RenderGraphError::EdgeAlreadyExists(edge));
        }
        self.output_edges.push(edge);
        Ok(())
    }

    /// Remove an edge from `output_edges` it exist.
    pub(crate) fn remove_output_edge(&mut self, edge: Edge) -> Result<(), RenderGraphError> {
        if let Some(index) = self.output_edges.iter().position(|e| *e == edge) {
            self.output_edges.swap_remove(index);
            Ok(())
        } else {
            Err(RenderGraphError::EdgeDoesNotExist(edge))
        }
    }

    /// Searches the `input_edges` for a [`Edge::SlotEdge`],
    /// which `input_index` matches the `index`;
    pub fn get_input_slot_edge(&self, index: usize) -> Result<&Edge, RenderGraphError> {
        self.input_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { input_index, .. } = e {
                    *input_index == index
                } else {
                    false
                }
            })
            .ok_or(RenderGraphError::UnconnectedNodeInputSlot {
                input_slot: index,
                node: self.id,
            })
    }

    /// Searches the `output_edges` for a [`Edge::SlotEdge`],
    /// which `output_index` matches the `index`;
    pub fn get_output_slot_edge(&self, index: usize) -> Result<&Edge, RenderGraphError> {
        self.output_edges
            .iter()
            .find(|e| {
                if let Edge::SlotEdge { output_index, .. } = e {
                    *output_index == index
                } else {
                    false
                }
            })
            .ok_or(RenderGraphError::UnconnectedNodeOutputSlot {
                output_slot: index,
                node: self.id,
            })
    }
}
