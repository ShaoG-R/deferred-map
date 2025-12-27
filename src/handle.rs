/// Handle is a one-time token for inserting values into DeferredMap
///
/// Handle does not implement the Clone trait, ensuring it can only be used once
/// through Rust's move semantics.
///
/// Handle 是一次性令牌，用于向 DeferredMap 插入值
///
/// Handle 不实现 Clone trait，通过 Rust 的 move semantics 确保只能使用一次
///
/// # Examples (示例)
///
/// ```
/// use deferred_map::DeferredMap;
///
/// let mut map = DeferredMap::new();
/// let handle = map.allocate_handle();
/// let key = handle.key();
/// map.insert(handle, 42);
/// assert_eq!(map.get(key), Some(&42));
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Handle {
    key: u64, // The key of the handle
}

impl Handle {
    /// Create a new Handle (internal use)
    ///
    /// 创建一个新的 Handle（内部使用）
    #[inline(always)]
    pub(crate) fn new(key: u64) -> Self {
        Self { key }
    }

    /// Get the key that will be used for this handle
    ///
    /// This is the same as raw_value(), but with a more semantic name.
    ///
    /// 获取此 handle 对应的 key
    ///
    /// 这与 raw_value() 相同，但名称更具语义性
    #[inline(always)]
    pub fn key(&self) -> u64 {
        self.key
    }

    /// Extract index (lower 32 bits)
    ///
    /// 提取 index（低 32 位）
    #[inline(always)]
    pub fn index(&self) -> u32 {
        self.key as u32
    }
    /// Extract generation (upper 32 bits)
    ///
    /// 提取 generation（高 32 位）
    #[inline(always)]
    pub fn generation(&self) -> u32 {
        (self.key >> 32) as u32
    }
}
