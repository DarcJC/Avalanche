use std::collections::HashMap;
use ash::vk;

/// Descriptor of a vertex attribute streaming into GPU.
#[derive(Debug)]
pub struct VertexStream {
    /// n + stride == address of next data
    stride: u32,
    /// type of this vertex data
    input_rate: vk::VertexInputRate,
    /// binding location specified by `layout(location = ?)`
    location: u32,
    /// a byte offset of this attribute relative to the start of an element in the vertex input binding
    offset: u32,
    /// attribute format
    format: vk::Format,
}

/// A group of [VertexStream]s.
pub struct VertexStreamSet {
    streams: Vec<VertexStream>,
    bindings: HashMap<u32, vk::VertexInputBindingDescription>,
}

impl VertexStreamSet {
    pub fn empty() -> Self {
        Self {
            streams: Vec::new(),
            bindings: HashMap::new(),
        }
    }

    pub fn add_stream(mut self, stride: u32, input_rate: vk::VertexInputRate, location: u32, format: vk::Format, offset: Option<u32>) -> Self {
        let offset = offset.unwrap_or(0);

        // find suitable binding
        let mut binding_index = None;
        for (index, binding) in self.bindings.iter() {
            if binding.stride == stride && binding.input_rate == input_rate {
                binding_index = Some(*index);
                break;
            }
        }

        // if binding index doesn't exist, create a new one
        let _binding_index = binding_index.unwrap_or_else(|| {
            let index = self.bindings.len() as u32;
            self.bindings.insert(index, vk::VertexInputBindingDescription::builder()
                .binding(index)
                .stride(stride)
                .input_rate(input_rate)
                .build()
            );
            index
        });

        let stream = VertexStream { stride, input_rate, location, offset, format };
        self.streams.push(stream);

        self
    }

    pub fn generate_description(&self) -> (Vec<vk::VertexInputBindingDescription>, Vec<vk::VertexInputAttributeDescription>) {
        let bindings = self.bindings.values().cloned().collect::<Vec<_>>();
        let mut attributes: Vec<vk::VertexInputAttributeDescription> = vec![];

        for stream in &self.streams {
            attributes.push(vk::VertexInputAttributeDescription::builder()
                .location(stream.location)
                .binding(*self
                    .bindings
                    .iter()
                    .find(|&(_, b)| b.stride == stream.stride && b.input_rate == stream.input_rate)
                    .unwrap()
                    .0)
                .offset(stream.offset)
                .format(stream.format)
                .build()
            );
        }

        (bindings, attributes)
    }
}
