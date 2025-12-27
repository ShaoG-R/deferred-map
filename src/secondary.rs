use crate::utils::unlikely;
use std::fmt;

/// Internal slot storage for SecondaryMap.
///
/// Stores the value and the generation it belongs to.
///
/// SecondaryMap 的内部 slot 存储。
/// 存储值及其所属的代数（generation）。
#[derive(Clone, Debug)]
struct Slot<T> {
    value: T,
    generation: u32,
}

/// A secondary map that associates data with keys from a `DeferredMap`.
///
/// `SecondaryMap` allows you to store additional information for each key in a `DeferredMap`.
/// It is separate from the primary map and does not affect the primary map's memory layout.
///
/// Key Features:
/// - **Sparse Storage**: Efficiently handles cases where not all keys in the primary map have associated data.
/// - **Generation Checking**: Automatically validates compatibility with the primary map's keys.
///   Stale keys (from older generations) will be ignored or overwritten as appropriate.
/// - **Automatic Expansion**: The map automatically grows to accommodate keys with larger indices.
///
///
/// 一个辅助映射（SecondaryMap），用于将数据与 `DeferredMap` 的 Key 关联。
///
/// `SecondaryMap` 允许你为 `DeferredMap` 中的每个 Key 存储额外信息。
/// 它与主映射分离，不影响主映射的内存布局。
///
/// 主要特性：
/// - **稀疏存储**：高效处理主映射中并非所有 Key 都有关联数据的情况。
/// - **代数检查**：自动验证与主映射 Key 的兼容性。过期的 Key（来自旧代数）将被忽略或覆盖。
/// - **自动扩展**：映射会自动增长以适应具有更大索引的 Key。
///
/// # Examples (示例)
///
/// ```
/// use deferred_map::{DeferredMap, SecondaryMap};
///
/// let mut map = DeferredMap::new();
/// let handle = map.allocate_handle();
/// let key = handle.key();
/// map.insert(handle, "Primary Value");
///
/// let mut sec_map = SecondaryMap::new();
/// sec_map.insert(key, 100);
///
/// assert_eq!(sec_map.get(key), Some(&100));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct SecondaryMap<T> {
    // We use Option to represent presence.
    // None means no value associated with this index for the stored generation.
    // Even if Some is present, we must check generation matching.
    slots: Vec<Option<Slot<T>>>,
    num_elems: usize,
    #[cfg(debug_assertions)]
    map_id: Option<u64>,
}

impl<T> Default for SecondaryMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> SecondaryMap<T> {
    /// Create a new empty SecondaryMap
    ///
    /// 创建一个新的空 SecondaryMap
    #[inline]
    pub fn new() -> Self {
        Self {
            slots: Vec::new(),
            num_elems: 0,
            #[cfg(debug_assertions)]
            map_id: None,
        }
    }

    /// Create a SecondaryMap with specified capacity
    ///
    /// 创建一个指定容量的 SecondaryMap
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            slots: Vec::with_capacity(capacity),
            num_elems: 0,
            #[cfg(debug_assertions)]
            map_id: None,
        }
    }

    /// Insert a value for a specific key
    ///
    /// If the key belongs to an older generation than what is currently stored, the insertion is ignored.
    /// If the key is newer, it overwrites the existing value.
    ///
    /// 为特定 Key 插入值
    ///
    /// 如果 Key 的代数（generation）老于当前存储的代数，插入将被忽略。
    /// 如果 Key 是新的，它将覆盖现有值。
    ///
    /// # Returns
    /// - `Some(old_value)` if a value existed for the EXACT same key (same index and generation).
    /// - `None` otherwise.
    pub fn insert(&mut self, key: crate::Key, value: T) -> Option<T> {
        #[cfg(debug_assertions)]
        {
            if let Some(id) = self.map_id {
                debug_assert_eq!(
                    id, key.map_id,
                    "Key used with wrong map instance in SecondaryMap"
                );
            } else {
                self.map_id = Some(key.map_id);
            }
        }

        let (index, generation) = key.decode();
        let index = index as usize;

        // Ensure we have enough slots
        // 确保有足够的 slot
        if index >= self.slots.len() {
            let required_additional = index - self.slots.len() + 1;
            self.slots.reserve(required_additional);
            self.slots.resize_with(index + 1, || None);
        }

        let slot_opt = unsafe { self.slots.get_unchecked_mut(index) };

        match slot_opt {
            Some(slot) => {
                if slot.generation == generation {
                    // Exact match, replace value
                    // 完全匹配，替换值
                    Some(std::mem::replace(&mut slot.value, value))
                } else if slot.generation < generation {
                    // Stale slot (older generation), overwrite with new data
                    // 槽位过期（旧代数），用新数据覆盖
                    // Note: We don't return the old value because it belongs to a different entity (older gen)
                    *slot = Slot { value, generation };
                    None
                } else {
                    // Incoming key is older than stored data, ignore insert
                    // 传入的 Key 比存储的数据旧，忽略插入
                    None
                }
            }
            None => {
                // Empty slot, insert new
                // 空槽位，插入新值
                *slot_opt = Some(Slot { value, generation });
                self.num_elems += 1;
                None
            }
        }
    }

    /// Remove value by key
    ///
    /// Only removes if both index and generation match.
    ///
    /// 通过 Key 移除值
    ///
    /// 仅当索引和代数都匹配时才移除。
    pub fn remove(&mut self, key: crate::Key) -> Option<T> {
        #[cfg(debug_assertions)]
        if let Some(id) = self.map_id {
            debug_assert_eq!(
                id, key.map_id,
                "Key used with wrong map instance in SecondaryMap"
            );
        }

        let (index, generation) = key.decode();
        let index = index as usize;

        if unlikely(index >= self.slots.len()) {
            return None;
        }

        let slot_opt = unsafe { self.slots.get_unchecked_mut(index) };

        if let Some(slot) = slot_opt {
            if slot.generation == generation {
                self.num_elems -= 1;
                return slot_opt.take().map(|s| s.value);
            }
        }

        None
    }

    /// Get reference to value by key
    ///
    /// 通过 Key 获取值的引用
    #[inline]
    pub fn get(&self, key: crate::Key) -> Option<&T> {
        #[cfg(debug_assertions)]
        if let Some(id) = self.map_id {
            debug_assert_eq!(
                id, key.map_id,
                "Key used with wrong map instance in SecondaryMap"
            );
        }

        let (index, generation) = key.decode();
        let index = index as usize;

        if unlikely(index >= self.slots.len()) {
            return None;
        }

        // SAFETY: Bounds checked above
        match unsafe { self.slots.get_unchecked(index) } {
            Some(slot) if slot.generation == generation => Some(&slot.value),
            _ => None,
        }
    }

    /// Get mutable reference to value by key
    ///
    /// 通过 Key 获取值的可变引用
    #[inline]
    pub fn get_mut(&mut self, key: crate::Key) -> Option<&mut T> {
        #[cfg(debug_assertions)]
        if let Some(id) = self.map_id {
            debug_assert_eq!(
                id, key.map_id,
                "Key used with wrong map instance in SecondaryMap"
            );
        }

        let (index, generation) = key.decode();
        let index = index as usize;

        if unlikely(index >= self.slots.len()) {
            return None;
        }

        // SAFETY: Bounds checked above
        match unsafe { self.slots.get_unchecked_mut(index) } {
            Some(slot) if slot.generation == generation => Some(&mut slot.value),
            _ => None,
        }
    }

    /// Check if key exists
    ///
    /// 检查 Key 是否存在
    #[inline]
    pub fn contains_key(&self, key: crate::Key) -> bool {
        self.get(key).is_some()
    }

    /// Return the number of elements
    ///
    /// 返回元素数量
    #[inline]
    pub fn len(&self) -> usize {
        self.num_elems
    }

    /// Check if empty
    ///
    /// 检查是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_elems == 0
    }

    /// Capacity of the underlying vector
    ///
    /// 底层 vector 的容量
    #[inline]
    pub fn capacity(&self) -> usize {
        self.slots.capacity()
    }

    /// Clear all elements
    ///
    /// Does not deallocate memory, but clears validity.
    ///
    /// 清空所有元素
    /// 不会释放内存，但会清除有效性。
    pub fn clear(&mut self) {
        self.slots.clear();
        self.num_elems = 0;
        #[cfg(debug_assertions)]
        {
            self.map_id = None;
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// 只保留满足谓词的元素。
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(crate::Key, &mut T) -> bool,
    {
        for (index, slot_opt) in self.slots.iter_mut().enumerate() {
            if let Some(slot) = slot_opt {
                let key = crate::Key::new(
                    index as u32,
                    slot.generation,
                    #[cfg(debug_assertions)]
                    self.map_id.unwrap_or(0),
                );

                if !f(key, &mut slot.value) {
                    *slot_opt = None;
                    self.num_elems -= 1;
                }
            }
        }
    }

    /// Iterator over all (key, value) pairs
    ///
    /// 遍历所有 (key, value) 对的迭代器
    pub fn iter(&self) -> impl Iterator<Item = (crate::Key, &T)> {
        self.slots
            .iter()
            .enumerate()
            .filter_map(move |(index, slot_opt)| {
                slot_opt.as_ref().map(|slot| {
                    let key = crate::Key::new(
                        index as u32,
                        slot.generation,
                        #[cfg(debug_assertions)]
                        self.map_id.unwrap_or(0),
                    );
                    (key, &slot.value)
                })
            })
    }

    /// Mutable iterator over all (key, value) pairs
    ///
    /// 遍历所有 (key, value) 对的可变迭代器
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (crate::Key, &mut T)> {
        #[cfg(debug_assertions)]
        let map_id = self.map_id;

        self.slots
            .iter_mut()
            .enumerate()
            .filter_map(move |(index, slot_opt)| {
                slot_opt.as_mut().map(|slot| {
                    let key = crate::Key::new(
                        index as u32,
                        slot.generation,
                        #[cfg(debug_assertions)]
                        map_id.unwrap_or(0),
                    );
                    (key, &mut slot.value)
                })
            })
    }
}

impl<T: fmt::Debug> fmt::Debug for SecondaryMap<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.iter()).finish()
    }
}
