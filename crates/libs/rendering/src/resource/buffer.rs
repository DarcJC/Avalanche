use std::ops::Deref;
use avalanche_utils::define_atomic_id_usize;
use crate::render_resource_wrapper;

define_atomic_id_usize!(BufferId);
render_resource_wrapper!(ErasedBuffer, avalanche_hlvk::Buffer);

#[derive(Clone, Debug)]
pub struct Buffer {
    id: BufferId,
    value: ErasedBuffer,
}

impl Buffer {
    #[inline]
    pub fn id(&self) -> BufferId {
        self.id
    }
}

impl From<avalanche_hlvk::Buffer> for Buffer {
    fn from(value: avalanche_hlvk::Buffer) -> Self {
        Buffer {
            id: BufferId::new(),
            value: ErasedBuffer::new(value),
        }
    }
}

impl Deref for Buffer {
    type Target = avalanche_hlvk::Buffer;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
