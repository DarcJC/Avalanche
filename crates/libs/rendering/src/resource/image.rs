use std::ops::Deref;
use avalanche_utils::define_atomic_id_usize;
use crate::render_resource_wrapper;

define_atomic_id_usize!(ImageId);
render_resource_wrapper!(ErasedImage, avalanche_hlvk::Image);

#[derive(Clone, Debug)]
pub struct Image {
    id: ImageId,
    value: ErasedImage,
}

impl Image {
    #[inline]
    pub fn id(&self) -> ImageId {
        self.id
    }
}

impl From<avalanche_hlvk::Image> for Image {
    fn from(value: avalanche_hlvk::Image) -> Self {
        Image {
            id: ImageId::new(),
            value: ErasedImage::new(value),
        }
    }
}

impl Deref for Image {
    type Target = avalanche_hlvk::Image;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

define_atomic_id_usize!(ImageViewId);
render_resource_wrapper!(ErasedImageView, avalanche_hlvk::ImageView);

#[derive(Clone, Debug)]
pub struct ImageView {
    id: ImageViewId,
    value: ErasedImageView,
}

impl ImageView {
    pub fn id(&self) -> ImageViewId {
        self.id
    }
}

impl From<avalanche_hlvk::ImageView> for ImageView {
    fn from(value: avalanche_hlvk::ImageView) -> Self {
        Self {
            id: ImageViewId::new(),
            value: ErasedImageView::new(value),
        }
    }
}

impl Deref for ImageView {
    type Target = avalanche_hlvk::ImageView;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

define_atomic_id_usize!(SamplerId);
render_resource_wrapper!(ErasedSampler, avalanche_hlvk::Sampler);

#[derive(Clone, Debug)]
pub struct Sampler {
    id: SamplerId,
    value: ErasedSampler,
}

impl Sampler {
    #[inline]
    pub fn id(&self) -> SamplerId {
        self.id
    }
}

impl From<avalanche_hlvk::Sampler> for Sampler {
    fn from(value: avalanche_hlvk::Sampler) -> Self {
        Self {
            id: SamplerId::new(),
            value: ErasedSampler::new(value),
        }
    }
}

impl Deref for Sampler {
    type Target = avalanche_hlvk::Sampler;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
