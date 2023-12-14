use ash::vk;
use avalanche_utils::Version;

pub(crate) trait IntoAshVersion {
    fn into_version(self) -> u32;
}

impl IntoAshVersion for Version {
    fn into_version(self) -> u32 {
        vk::make_api_version(self.variant, self.major, self.minor, self.patch)
    }
}
