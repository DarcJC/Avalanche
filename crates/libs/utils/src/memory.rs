
pub fn compute_aligned_size(size: u32, alignment: u32) -> u32 {
    (size + (alignment - 1)) & !(alignment - 1)
}
