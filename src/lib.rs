mod handle;
mod map;
mod secondary;
mod slot;
mod utils;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Key {
    pub(crate) raw: u64,
    #[cfg(debug_assertions)]
    pub(crate) map_id: u64,
}

impl Key {
    /// Create a new Key from index and generation
    ///
    /// 从 index 和 generation 创建新的 Key
    #[inline(always)]
    pub fn new(index: u32, generation: u32, #[cfg(debug_assertions)] map_id: u64) -> Self {
        Self {
            raw: ((generation as u64) << 32) | (index as u64),
            #[cfg(debug_assertions)]
            map_id,
        }
    }

    /// Decode Key into index and generation
    ///
    /// 将 Key 解码为 index 和 generation
    #[inline(always)]
    pub fn decode(&self) -> (u32, u32) {
        (self.index(), self.generation())
    }

    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.raw as u32
    }

    #[inline(always)]
    pub fn generation(&self) -> u32 {
        (self.raw >> 32) as u32
    }
}

pub use handle::Handle;
pub use map::DeferredMap;
pub use secondary::SecondaryMap;

#[cfg(test)]
mod tests {
    // Test modules for DeferredMap
    // DeferredMap 的测试模块
    mod debug_safety;
    mod edge_cases;
    mod handle;
    mod insertion;
    mod new_features;
    mod removal;
    mod secondary_test;
}
