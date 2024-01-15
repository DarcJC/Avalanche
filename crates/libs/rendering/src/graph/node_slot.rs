use core::fmt;
use std::borrow::Cow;
use std::fmt::Formatter;
use bevy_ecs::prelude::Entity;
use crate::resource::{Buffer, ImageView, Sampler};

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum SlotType {
    Buffer,
    ImageView,
    Sampler,
    Entity,
}

impl fmt::Display for SlotType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use SlotType::*;
        let s = match self {
            Buffer => "Buffer",
            ImageView => "ImageView",
            Sampler => "Sampler",
            Entity => "Entity",
        };

        f.write_str(s)
    }
}

#[derive(Clone, Debug)]
pub enum SlotValue {
    /// A GPU-accessible [`Buffer`].
    Buffer(Buffer),
    /// An [`ImageView`] describes the usage of an [`Image`] which also named as the texture.
    ImageView(ImageView),
    /// A texture [`Sampler`] defines the pipeline state to sample a [`ImageView`].
    Sampler(Sampler),
    /// An entity in render ECS world.
    Entity(Entity),
}

impl SlotValue {
    pub fn slot_type(&self) -> SlotType {
        use SlotValue::*;

        match self {
            Buffer(_) => SlotType::Buffer,
            ImageView(_) => SlotType::ImageView,
            Sampler(_) => SlotType::Sampler,
            Entity(_) => SlotType::Entity,
        }
    }
}

impl From<Buffer> for SlotValue {
    fn from(value: Buffer) -> Self {
        SlotValue::Buffer(value)
    }
}

impl From<ImageView> for SlotValue {
    fn from(value: ImageView) -> Self {
        SlotValue::ImageView(value)
    }
}

impl From<Sampler> for SlotValue {
    fn from(value: Sampler) -> Self {
        SlotValue::Sampler(value)
    }
}

impl From<Entity> for SlotValue {
    fn from(value: Entity) -> Self {
        SlotValue::Entity(value)
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SlotLabel {
    Index(usize),
    Name(Cow<'static, str>),
}

impl From<&SlotLabel> for SlotLabel {
    fn from(value: &SlotLabel) -> Self {
        value.clone()
    }
}

impl From<String> for SlotLabel {
    fn from(value: String) -> Self {
        SlotLabel::Name(value.into())
    }
}

impl From<&'static str> for SlotLabel {
    fn from(value: &'static str) -> Self {
        SlotLabel::Name(value.into())
    }
}

impl From<Cow<'static, str>> for SlotLabel {
    fn from(value: Cow<'static, str>) -> Self {
        SlotLabel::Name(value)
    }
}

impl From<usize> for SlotLabel {
    fn from(value: usize) -> Self {
        SlotLabel::Index(value)
    }
}

#[derive(Clone, Debug)]
pub struct SlotInfo {
    pub name: Cow<'static, str>,
    pub slot_type: SlotType,
}

impl SlotInfo {
    pub fn new(name: impl Into<Cow<'static, str>>, slot_type: SlotType) -> Self {
        SlotInfo {
            name: name.into(),
            slot_type,
        }
    }
}

#[derive(Default, Debug)]
pub struct SlotInfos {
    slots: Vec<SlotInfo>,
}

impl<T: IntoIterator<Item = SlotInfo>> From<T> for SlotInfos {
    fn from(value: T) -> Self {
        SlotInfos {
            slots: value.into_iter().collect::<Vec<_>>(),
        }
    }
}

impl SlotInfos {
    #[inline]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    pub fn get_slot(&self, label: impl Into<SlotLabel>) -> Option<&SlotInfo> {
        let label = label.into();
        let index = self.get_slot_index(label)?;
        self.slots.get(index)
    }

    pub fn get_slot_mut(&mut self, label: impl Into<SlotLabel>) -> Option<&mut SlotInfo> {
        let label = label.into();
        let index = self.get_slot_index(label)?;
        self.slots.get_mut(index)
    }

    /// Get slot index of the provided label.
    ///
    /// When passing a usize index manually,
    /// the index must less than the [`SlotInfos::len`] of this slots.
    ///
    /// If index is invalid or could not found slot with name,
    /// this function will return a [`None`].
    pub fn get_slot_index(&self, label: impl Into<SlotLabel>) -> Option<usize> {
        let label = label.into();
        match label {
            SlotLabel::Index(index) => {
                if index < self.len() {
                    Some(index)
                } else {
                    None
                }
            },
            SlotLabel::Name(ref name) => self.slots.iter().position(|s| s.name == *name)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &SlotInfo> {
        self.slots.iter()
    }
}
