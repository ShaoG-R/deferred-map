use crate::handle::Handle;
use crate::slot::SlotContent::Occupied;
use crate::slot::SlotContentMut::OccupiedMut;
use crate::slot::{Slot, SlotUnion};
use crate::utils::{decode_key, encode_key, likely, unlikely};
use std::mem::ManuallyDrop;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(debug_assertions)]
static NEXT_MAP_ID: AtomicU64 = AtomicU64::new(0);

/// DeferredMap is a high-performance map based on slotmap
///
/// Usage requires first obtaining a Handle via `allocate_handle`,
/// then using the Handle to insert.
///
/// DeferredMap 是一个基于 slotmap 的高性能映射表
///
/// 使用前需要先通过 `allocate_handle` 获取 Handle，然后使用 Handle 进行插入
///
/// # Features (特性)
///
/// - O(1) insertion, lookup, and removal | O(1) 插入、查找和删除
/// - Generational indices prevent use-after-free | 代数索引防止释放后使用
/// - Handle-based deferred insertion | 基于 Handle 的延迟插入
/// - Memory efficient with union-based slots | 使用 union 的内存高效 slot
///
/// # Examples (示例)
///
/// ```
/// use deferred_map::DeferredMap;
///
/// let mut map = DeferredMap::new();
///
/// // Allocate handle first | 先分配 handle
/// let handle = map.allocate_handle();
/// let key = handle.key();
///
/// // Insert value later | 之后插入值
/// map.insert(handle, 42);
///
/// // Access value | 访问值
/// assert_eq!(map.get(key), Some(&42));
///
/// // Remove value | 删除值
/// assert_eq!(map.remove(key), Some(42));
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeferredMap<T> {
    slots: Vec<Slot<T>>,
    free_head: u32, // Head of free list | 空闲列表的头部索引
    num_elems: u32, // Current element count | 当前元素数量
    #[cfg(debug_assertions)]
    map_id: u64,
}

impl<T> DeferredMap<T> {
    /// Create a new empty DeferredMap
    ///
    /// 创建一个新的空 DeferredMap
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let map: DeferredMap<i32> = DeferredMap::new();
    /// assert!(map.is_empty());
    /// ```
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Create a DeferredMap with specified capacity
    ///
    /// 创建一个指定容量的 DeferredMap
    ///
    /// # Parameters
    /// - `capacity`: Initial capacity (number of slots to pre-allocate)
    ///
    /// # 参数
    /// - `capacity`: 初始容量（预分配的 slot 数量）
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let map: DeferredMap<i32> = DeferredMap::with_capacity(100);
    /// assert_eq!(map.capacity(), 0);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // Create slots with sentinel at index 0
        // Sentinel is not used but maintains index consistency
        // 创建 slots，在索引 0 处添加 sentinel
        // sentinel 不实际使用，但保持索引一致性
        let mut slots = Vec::with_capacity(capacity + 1);
        slots.push(Slot {
            u: SlotUnion { next_free: 0 },
            version: 0,
        });

        Self {
            slots,
            free_head: 1, // Start allocation from index 1 | 从索引 1 开始分配
            num_elems: 0,
            #[cfg(debug_assertions)]
            map_id: NEXT_MAP_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    /// Pre-allocate a Handle
    ///
    /// This Handle can be used later to insert a value.
    /// The slot is immediately created in Reserved state.
    ///
    /// 预分配一个 Handle
    ///
    /// 这个 Handle 可以在之后用于插入值
    /// slot 立即创建为 Reserved 状态
    ///
    /// # Returns
    /// A unique Handle for later insertion
    ///
    /// # 返回值
    /// 用于后续插入的唯一 Handle
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::new();
    /// let handle = map.allocate_handle();
    /// let key = handle.key();
    /// map.insert(handle, "value");
    /// assert_eq!(map.get(key), Some(&"value"));
    /// ```
    pub fn allocate_handle(&mut self) -> Handle {
        if let Some(slot) = self.slots.get_mut(self.free_head as usize) {
            // Reuse existing vacant slot from free list
            // 从空闲列表中复用已有的空闲 slot
            let index = self.free_head;

            // Update free_head to next free slot before changing state
            // 在改变状态前更新 free_head 到下一个空闲 slot
            unsafe {
                self.free_head = slot.u.next_free;
            }

            // Transition: vacant(0bXX00) -> reserved(0bXX01)
            // 状态转换：vacant(0bXX00) -> reserved(0bXX01)
            slot.version += 1;

            let raw = encode_key(index, slot.generation());
            Handle::new(
                raw,
                #[cfg(debug_assertions)]
                self.map_id,
            )
        } else {
            // Need to extend Vec, allocate new slot
            // 需要扩展 Vec，分配新 slot
            let index = self.slots.len() as u32;
            let version = 0b01; // New slot starts in reserved state | 新 slot 从 reserved 状态开始

            // Create reserved slot
            // 创建 reserved slot
            self.slots.push(Slot {
                u: SlotUnion { next_free: 0 }, // Value doesn't matter for reserved | 对于 reserved 状态该值不重要
                version,
            });

            // Update free_head
            // 更新 free_head
            self.free_head = index + 1;

            // Extract generation from version (reserved state: 0b01)
            // 从 version 提取 generation（reserved 状态：0b01）
            let raw = encode_key(index, version >> 2);
            Handle::new(
                raw,
                #[cfg(debug_assertions)]
                self.map_id,
            )
        }
    }

    /// Insert value using Handle
    ///
    /// The Handle is consumed (moved), ensuring it can only be used once.
    /// The slot must be in Reserved state.
    ///
    /// 使用 Handle 插入值
    ///
    /// Handle 会被消耗（move），确保只能使用一次
    /// slot 必须处于 Reserved 状态
    ///
    /// # Parameters
    /// - `handle`: The Handle obtained from `allocate_handle`
    /// - `value`: The value to insert
    ///
    /// # 参数
    /// - `handle`: 从 `allocate_handle` 获取的 Handle
    /// - `value`: 要插入的值
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
    pub fn insert(&mut self, handle: Handle, value: T) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            self.map_id, handle.map_id,
            "Handle used with wrong map instance"
        );

        let index = handle.index();

        // Validate index (skip sentinel)
        // 验证 index 有效（跳过 sentinel）
        debug_assert!(index != 0, "Invalid handle: sentinel index");

        // Slot must exist (allocate_handle should have created it)
        // slot 必须存在（allocate_handle 应该已经创建了它）
        debug_assert!(
            (index as usize) < self.slots.len(),
            "Invalid handle: index out of bounds"
        );

        let slot = unsafe { self.slots.get_unchecked_mut(index as usize) };

        // Validate generation match (handle stores generation, not version)
        // 验证 generation 匹配（handle 存储 generation，不是 version）
        debug_assert!(
            slot.generation() == handle.generation(),
            "Generation mismatch"
        );

        // Validate slot is in Reserved state
        // 验证 slot 处于 Reserved 状态
        debug_assert!(slot.is_reserved(), "Handle already used or invalid state");

        // Insert value and transition: reserved(0bXX01) -> occupied(0bXX11)
        // 插入值并状态转换：reserved(0bXX01) -> occupied(0bXX11)
        slot.u.value = ManuallyDrop::new(value);
        slot.version += 2; // 0bXX01 -> 0bXX11

        self.num_elems += 1;
    }

    /// Get immutable reference to value by u64 key
    ///
    /// 通过 u64 key 获取值的不可变引用
    ///
    /// # Parameters
    /// - `key`: The key returned from `insert`
    ///
    /// # Returns
    /// - `Some(&T)`: Reference to the value if key is valid
    /// - `None`: If key is invalid or value has been removed
    ///
    /// # 参数
    /// - `key`: 从 `insert` 返回的 key
    ///
    /// # 返回值
    /// - `Some(&T)`: 如果 key 有效则返回值的引用
    /// - `None`: 如果 key 无效或值已被删除
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
    #[inline]
    pub fn get(&self, key: u64) -> Option<&T> {
        let (index, generation) = decode_key(key);

        // Bounds check
        // 边界检查
        if unlikely(index as usize >= self.slots.len()) {
            return None;
        }

        // SAFETY: We've checked that index < slots.len()
        let slot = unsafe { self.slots.get_unchecked(index as usize) };

        // Fast path: check generation match and return value
        // 快速路径：检查 generation 匹配并返回值
        if likely(slot.generation() == generation && slot.is_occupied()) {
            // SAFETY: We've checked that slot is occupied
            Some(unsafe { &*slot.u.value })
        } else {
            None
        }
    }

    /// Get mutable reference to value by u64 key
    ///
    /// 通过 u64 key 获取值的可变引用
    ///
    /// # Parameters
    /// - `key`: The key returned from `insert`
    ///
    /// # Returns
    /// - `Some(&mut T)`: Mutable reference to the value if key is valid
    /// - `None`: If key is invalid or value has been removed
    ///
    /// # 参数
    /// - `key`: 从 `insert` 返回的 key
    ///
    /// # 返回值
    /// - `Some(&mut T)`: 如果 key 有效则返回值的可变引用
    /// - `None`: 如果 key 无效或值已被删除
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
    ///
    /// if let Some(value) = map.get_mut(key) {
    ///     *value = 100;
    /// }
    /// assert_eq!(map.get(key), Some(&100));
    /// ```
    #[inline]
    pub fn get_mut(&mut self, key: u64) -> Option<&mut T> {
        let (index, generation) = decode_key(key);

        // Bounds check
        // 边界检查
        if unlikely(index as usize >= self.slots.len()) {
            return None;
        }

        // SAFETY: We've checked that index < slots.len()
        let slot = unsafe { self.slots.get_unchecked_mut(index as usize) };

        // Fast path: check generation match and return mutable reference
        // 快速路径：检查 generation 匹配并返回可变引用
        if likely(slot.generation() == generation && slot.is_occupied()) {
            // SAFETY: We've checked that slot is occupied
            Some(unsafe { &mut *slot.u.value })
        } else {
            None
        }
    }

    /// Remove value by u64 key
    ///
    /// If successful, returns the removed value and adds the slot to the free list.
    ///
    /// 通过 u64 key 移除值
    ///
    /// 如果成功移除，返回被移除的值，并将该 slot 加入空闲列表
    ///
    /// # Parameters
    /// - `key`: The key returned from `insert`
    ///
    /// # Returns
    /// - `Some(T)`: The removed value if key is valid
    /// - `None`: If key is invalid or value has already been removed
    ///
    /// # 参数
    /// - `key`: 从 `insert` 返回的 key
    ///
    /// # 返回值
    /// - `Some(T)`: 如果 key 有效则返回被移除的值
    /// - `None`: 如果 key 无效或值已被删除
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
    ///
    /// assert_eq!(map.remove(key), Some(42));
    /// assert_eq!(map.get(key), None);
    /// ```
    #[inline]
    pub fn remove(&mut self, key: u64) -> Option<T> {
        let (index, generation) = decode_key(key);

        // Bounds check
        // 边界检查
        if unlikely(index as usize >= self.slots.len()) {
            return None;
        }

        // SAFETY: We've checked that index < slots.len()
        let slot = unsafe { self.slots.get_unchecked_mut(index as usize) };

        // Fast path: check generation and occupied state
        // 快速路径：检查 generation 和占用状态
        if likely(slot.generation() == generation && slot.is_occupied()) {
            // Take value from slot
            // 从 slot 中取出值
            let value = unsafe { ManuallyDrop::take(&mut slot.u.value) };

            // Add this slot to free list head
            // 将此 slot 加入空闲列表头部
            slot.u.next_free = self.free_head;
            self.free_head = index;

            // Transition: occupied(0bXX11) -> vacant(0bYY00, next generation)
            // 状态转换：occupied(0bXX11) -> vacant(0bYY00，下一代）
            slot.version = slot.version.wrapping_add(1); // 0bXX11 -> 0bYY00

            self.num_elems -= 1;
            Some(value)
        } else {
            None
        }
    }

    /// Release an unused Handle
    ///
    /// Returns the reserved slot back to the free list.
    /// This is useful when you allocated a handle but decided not to use it.
    ///
    /// 释放未使用的 Handle
    ///
    /// 将预留的 slot 返回到空闲列表
    /// 当你分配了 handle 但决定不使用时很有用
    ///
    /// # Parameters
    /// - `handle`: The Handle to release
    ///
    /// # 参数
    /// - `handle`: 要释放的 Handle
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::<i32>::new();
    /// let handle = map.allocate_handle();
    ///
    /// // Decided not to use it
    /// // 决定不使用它
    /// map.release_handle(handle);
    /// ```
    pub fn release_handle(&mut self, handle: Handle) {
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            self.map_id, handle.map_id,
            "Handle used with wrong map instance"
        );

        let index = handle.index();
        let handle_generation = handle.generation();

        // Validate index (skip sentinel)
        // 验证 index 有效（跳过 sentinel）
        debug_assert!(index != 0, "Invalid handle: sentinel index");

        // Slot must exist
        // slot 必须存在
        debug_assert!(
            (index as usize) < self.slots.len(),
            "Invalid handle: index out of bounds"
        );

        let slot = &mut self.slots[index as usize];

        // Validate generation match (handle stores generation, not version)
        // 验证 generation 匹配（handle 存储 generation，不是 version）
        debug_assert!(
            slot.generation() == handle_generation,
            "Generation mismatch"
        );

        // Validate slot is in Reserved state
        // 验证 slot 处于 Reserved 状态
        debug_assert!(slot.is_reserved(), "Handle already used or invalid state");

        // Add this slot to free list head
        // 将此 slot 加入空闲列表头部
        slot.u.next_free = self.free_head;
        self.free_head = index;

        // Transition: reserved(0bXX01) -> vacant(0bYY00, next generation)
        // 状态转换：reserved(0bXX01) -> vacant(0bYY00，下一代）
        slot.version = slot.version.wrapping_add(3); // 0bXX01 + 3 = 0bYY00
    }

    /// Check if key exists
    ///
    /// 检查 key 是否存在
    ///
    /// # Parameters
    /// - `key`: The key to check
    ///
    /// # Returns
    /// `true` if the key exists, `false` otherwise
    ///
    /// # 参数
    /// - `key`: 要检查的 key
    ///
    /// # 返回值
    /// 如果 key 存在则返回 `true`，否则返回 `false`
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
    ///
    /// assert!(map.contains_key(key));
    /// map.remove(key);
    /// assert!(!map.contains_key(key));
    /// ```
    #[inline]
    pub fn contains_key(&self, key: u64) -> bool {
        self.get(key).is_some()
    }

    /// Return the number of valid elements
    ///
    /// 返回有效元素的数量
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::new();
    /// assert_eq!(map.len(), 0);
    ///
    /// let handle = map.allocate_handle();
    /// map.insert(handle, 42);
    /// assert_eq!(map.len(), 1);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.num_elems as usize
    }

    /// Check if the map is empty
    ///
    /// 检查是否为空
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let map: DeferredMap<i32> = DeferredMap::new();
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.num_elems == 0
    }

    /// Return capacity (number of allocated slots, excluding sentinel)
    ///
    /// 返回容量（已分配的 slot 数量，不包括 sentinel）
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let map: DeferredMap<i32> = DeferredMap::with_capacity(10);
    /// assert_eq!(map.capacity(), 0);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        // Subtract sentinel slot
        // 减去 sentinel slot
        self.slots.len().saturating_sub(1)
    }

    /// Clear all elements
    ///
    /// 清空所有元素
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::new();
    /// let handle = map.allocate_handle();
    /// map.insert(handle, 42);
    ///
    /// map.clear();
    /// assert!(map.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.slots.clear();
        // Re-add sentinel
        // 重新添加 sentinel
        self.slots.push(Slot {
            u: SlotUnion { next_free: 0 },
            version: 0,
        });
        self.free_head = 1;
        self.num_elems = 0;
    }

    /// Return an iterator over all (key, value) pairs
    ///
    /// 返回一个迭代器，遍历所有 (key, value) 对
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::new();
    ///
    /// let h1 = map.allocate_handle();
    /// map.insert(h1, 1);
    ///
    /// let h2 = map.allocate_handle();
    /// map.insert(h2, 2);
    ///
    /// let sum: i32 = map.iter().map(|(_, v)| v).sum();
    /// assert_eq!(sum, 3);
    /// ```
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = (u64, &T)> {
        self.slots
            .iter()
            .enumerate()
            .skip(1)
            .filter_map(|(index, slot)| {
                if let Occupied(value) = slot.get() {
                    let key = encode_key(index as u32, slot.generation());
                    Some((key, value))
                } else {
                    None
                }
            })
    }

    /// Return a mutable iterator over all (key, value) pairs
    ///
    /// 返回一个可变迭代器，遍历所有 (key, value) 对
    ///
    /// # Examples (示例)
    ///
    /// ```
    /// use deferred_map::DeferredMap;
    ///
    /// let mut map = DeferredMap::new();
    ///
    /// let h1 = map.allocate_handle();
    /// map.insert(h1, 1);
    ///
    /// let h2 = map.allocate_handle();
    /// map.insert(h2, 2);
    ///
    /// for (_, value) in map.iter_mut() {
    ///     *value *= 2;
    /// }
    ///
    /// let sum: i32 = map.iter().map(|(_, v)| v).sum();
    /// assert_eq!(sum, 6);
    /// ```
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (u64, &mut T)> {
        self.slots
            .iter_mut()
            .enumerate()
            .skip(1)
            .filter_map(|(index, slot)| {
                let generation = slot.generation();
                if let OccupiedMut(value) = slot.get_mut() {
                    let key = encode_key(index as u32, generation);
                    Some((key, value))
                } else {
                    None
                }
            })
    }
}

impl<T: Clone> Clone for DeferredMap<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            slots: self.slots.clone(),
            free_head: self.free_head,
            num_elems: self.num_elems,
            #[cfg(debug_assertions)]
            map_id: NEXT_MAP_ID.fetch_add(1, Ordering::Relaxed),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        self.slots.clone_from(&source.slots);
        self.free_head = source.free_head;
        self.num_elems = source.num_elems;
        #[cfg(debug_assertions)]
        {
            self.map_id = NEXT_MAP_ID.fetch_add(1, Ordering::Relaxed);
        }
    }
}

impl<T> Default for DeferredMap<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_basic_insert_and_get() {
        let mut map = DeferredMap::new();

        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, 42);

        assert_eq!(map.get(key), Some(&42));
    }

    #[test]
    fn test_remove_and_reuse() {
        let mut map = DeferredMap::new();

        let handle1 = map.allocate_handle();
        let key1 = handle1.key();
        map.insert(handle1, 42);

        assert_eq!(map.len(), 1);
        assert_eq!(map.remove(key1), Some(42));
        assert_eq!(map.len(), 0);
        assert_eq!(map.get(key1), None);

        // Allocating new handle should reuse previous slot
        // 分配新的 handle 应该复用之前的 slot
        let handle2 = map.allocate_handle();
        let key2 = handle2.key();
        map.insert(handle2, 100);

        // key2 should have different generation
        // key2 应该有不同的 generation
        assert_ne!(key1, key2);
        assert_eq!(map.get(key2), Some(&100));
        assert_eq!(map.get(key1), None); // Old key should be invalid | 旧 key 应该无效
    }

    #[test]
    fn test_multiple_inserts() {
        let mut map = DeferredMap::new();

        let mut keys = Vec::new();
        for i in 0..10 {
            let handle = map.allocate_handle();
            let key = handle.key();
            map.insert(handle, i * 10);
            keys.push(key);
        }

        assert_eq!(map.len(), 10);

        for (i, &key) in keys.iter().enumerate() {
            assert_eq!(map.get(key), Some(&(i * 10)));
        }
    }

    #[test]
    fn test_get_mut() {
        let mut map = DeferredMap::new();

        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, 42);

        if let Some(value) = map.get_mut(key) {
            *value = 100;
        }

        assert_eq!(map.get(key), Some(&100));
    }

    #[test]
    fn test_contains_key() {
        let mut map = DeferredMap::new();

        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, 42);

        assert!(map.contains_key(key));

        map.remove(key);
        assert!(!map.contains_key(key));
    }

    #[test]
    fn test_is_empty() {
        let mut map: DeferredMap<i32> = DeferredMap::new();

        assert!(map.is_empty());

        let handle = map.allocate_handle();
        let key = handle.key();
        map.insert(handle, 42);

        assert!(!map.is_empty());

        map.remove(key);
        assert!(map.is_empty());
    }

    #[test]
    fn test_capacity() {
        let mut map: DeferredMap<i32> = DeferredMap::with_capacity(10);

        for _ in 0..5 {
            let handle = map.allocate_handle();
            map.insert(handle, 42);
        }

        assert_eq!(map.len(), 5);
        assert_eq!(map.capacity(), 5);
    }

    #[test]
    fn test_clear() {
        let mut map = DeferredMap::new();

        for i in 0..5 {
            let handle = map.allocate_handle();
            map.insert(handle, i);
        }

        assert_eq!(map.len(), 5);

        map.clear();

        assert_eq!(map.len(), 0);
        assert_eq!(map.capacity(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn test_iter() {
        let mut map = DeferredMap::new();

        let mut keys = Vec::new();
        for i in 0..5 {
            let handle = map.allocate_handle();
            let key = handle.key();
            map.insert(handle, i * 10);
            keys.push(key);
        }

        let collected: Vec<_> = map.iter().collect();
        assert_eq!(collected.len(), 5);

        for (key, &value) in map.iter() {
            assert!(keys.contains(&key));
            let index = keys.iter().position(|&k| k == key).unwrap();
            assert_eq!(value, index * 10);
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut map = DeferredMap::new();

        for i in 0..5 {
            let handle = map.allocate_handle();
            map.insert(handle, i);
        }

        for (_, value) in map.iter_mut() {
            *value *= 2;
        }

        for (_, &value) in map.iter() {
            assert_eq!(value % 2, 0);
        }
    }

    #[test]
    fn test_handle_encoding_decoding() {
        let mut map: DeferredMap<i32> = DeferredMap::new();
        let handle = map.allocate_handle();

        let key = handle.key();
        let index = handle.index();
        let generation = handle.generation();

        // encode_key uses generation (32 bits), not version (which includes state bits)
        // encode_key 使用 generation（32 位），而不是 version（包含状态位）
        assert_eq!(encode_key(index, generation), key);
        assert_eq!(decode_key(key), (index, generation));
    }

    #[test]
    fn test_stress_test() {
        let mut map = DeferredMap::new();
        let mut keys = Vec::new();

        // Insert 100 elements | 插入 100 个元素
        for i in 0..100 {
            let handle = map.allocate_handle();
            let key = handle.key();
            map.insert(handle, i);
            keys.push(key);
        }

        assert_eq!(map.len(), 100);

        // Remove elements at even indices | 删除偶数索引的元素
        for i in (0..100).step_by(2) {
            map.remove(keys[i]);
        }

        assert_eq!(map.len(), 50);

        // Re-insert 50 elements (should reuse previously deleted slots)
        // 重新插入 50 个元素（应该复用之前删除的 slot）
        for i in 0..50 {
            let handle = map.allocate_handle();
            let key = handle.key();
            map.insert(handle, i + 1000);
            keys[i * 2] = key; // Update key | 更新 key
        }

        assert_eq!(map.len(), 100);

        // Verify all elements | 验证所有元素
        let mut count = 0;
        for (_, _) in map.iter() {
            count += 1;
        }
        assert_eq!(count, 100);
    }

    #[test]
    fn test_generation_wrapping() {
        let mut map = DeferredMap::new();

        // Test generation wrapping
        // Through many insertions and deletions to increment version
        // 测试 generation wrapping
        // 通过大量的插入和删除来增加 version
        let mut keys = Vec::new();
        for i in 0..10 {
            let handle = map.allocate_handle();
            let key = handle.key();
            map.insert(handle, i);
            keys.push(key);
        }

        // Remove all, test version increment
        // 删除所有，测试 version 递增
        for key in &keys {
            map.remove(*key);
        }

        // Re-insert, version should increment
        // 重新插入，version 应该递增
        let handle = map.allocate_handle();
        let new_key = handle.key();
        map.insert(handle, 100);

        // Old key should be invalid | 旧 key 应该无效
        assert_eq!(map.get(keys[0]), None);
        // New key is valid | 新 key 有效
        assert_eq!(map.get(new_key), Some(&100));
    }
}
