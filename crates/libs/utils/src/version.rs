use crate::const_compute::parse_unwarp;

pub const VERSION_1_0: Version = Version::from_major_minor(1, 0);
pub const VERSION_1_1: Version = Version::from_major_minor(1, 1);
pub const VERSION_1_2: Version = Version::from_major_minor(1, 2);
pub const VERSION_1_3: Version = Version::from_major_minor(1, 3);

pub const CURRENT_APPLICATION_VERSION: Version = Version::new(0, parse_unwarp(env!("CARGO_PKG_VERSION_MAJOR")) as u32, parse_unwarp(env!("CARGO_PKG_VERSION_MINOR")) as u32, parse_unwarp(env!("CARGO_PKG_VERSION_PATCH")) as u32);

#[derive(Debug, Clone, Copy)]
pub struct Version {
    pub variant: u32,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Version {
    pub const fn new(variant: u32, major: u32, minor: u32, patch: u32) -> Self {
        Self {
            variant,
            major,
            minor,
            patch,
        }
    }

    pub const fn from_major(major: u32) -> Self {
        Self {
            major,
            ..Self::default()
        }
    }

    pub const fn from_major_minor(major: u32, minor: u32) -> Self {
        Self {
            major,
            minor,
            ..Self::default()
        }
    }

    const fn default() -> Self {
        Self {
            variant: 0,
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
}
