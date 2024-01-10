use std::borrow::Cow;
use bevy_utils::thiserror::Error;
use crate::prelude::node_slot::{SlotLabel, SlotType};


#[derive(Error, Debug, Eq, PartialEq)]
pub enum NodeRunError {
    #[error("encountered an input slot error")]
    InputSlotError(#[from] InputSlotError),
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum RunSubGraphError {
    #[error("attempted to run sub-graph `{0}`, but it does not exist")]
    MissingSubGraph(Cow<'static, str>),
    #[error("attempted to pass inputs to sub-graph `{0}`, which has no input slots")]
    SubGraphHasNoInputs(Cow<'static, str>),
    #[error("sub graph (name: `{graph_name:?}`) could not be run because slot `{slot_name}` at index {slot_index} has no value")]
    MissingInput {
        slot_index: usize,
        slot_name: Cow<'static, str>,
        graph_name: Cow<'static, str>,
    },
    #[error("attempted to use the wrong type for input slot")]
    MismatchedInputSlotType {
        graph_name: Cow<'static, str>,
        slot_index: usize,
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum OutputSlotError {
    #[error("output slot `{0:?}` does not exist")]
    InvalidSlot(SlotLabel),
    #[error("attempted to output a value of type `{actual}` to output slot `{label:?}`, which has type `{expected}`")]
    MismatchedSlotType {
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}

#[derive(Error, Debug, Eq, PartialEq)]
pub enum InputSlotError {
    #[error("input slot `{0:?}` does not exist")]
    InvalidSlot(SlotLabel),
    #[error("attempted to retrieve a value of type `{actual}` from input slot `{label:?}`, which has type `{expected}`")]
    MismatchedSlotType {
        label: SlotLabel,
        expected: SlotType,
        actual: SlotType,
    },
}
