use downcast_rs::{Downcast, impl_downcast};
use avalanche_utils::define_atomic_id;

define_atomic_id!(NodeId);

pub trait Node: Downcast + Send + Sync + 'static {
}
impl_downcast!(Node);
