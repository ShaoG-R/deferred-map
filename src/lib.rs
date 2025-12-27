mod handle;
mod map;
mod secondary;
mod slot;
mod utils;

use std::fmt::Debug;
use std::hash::Hash;

use std::fmt;
use std::num::NonZeroU32;
use std::ops::Deref;

use crate::utils::unlikely;

/// A generation counter that is always non-zero.
///
/// Used for preventing ABA problems in slot re-use.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Generation(NonZeroU32);

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Generation {
    pub const MIN: Self = unsafe { Self(NonZeroU32::new_unchecked(1)) };

    #[inline(always)]
    #[cfg(feature = "serde")]
    pub const fn new(val: NonZeroU32) -> Self {
        Self(val)
    }

    #[inline(always)]
    pub const unsafe fn new_unchecked(val: u32) -> Self {
        unsafe { Self(NonZeroU32::new_unchecked(val)) }
    }
}

impl Deref for Generation {
    type Target = NonZeroU32;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Represents the version of a slot, combining generation and state.
///
/// The lowest 2 bits represent the state:
/// - 0b00: Vacant
/// - 0b01: Reserved
/// - 0b11: Occupied
///
/// The upper 30 bits represent the generation.
///
/// 表示 slot 的版本，结合了代数（generation）和状态。
///
/// 最低 2 位表示状态：
/// - 0b00: 空闲 (Vacant)
/// - 0b01: 预留 (Reserved)
/// - 0b11: 占用 (Occupied)
///
/// 高 30 位表示代数。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Version(NonZeroU32);

impl Version {
    /// Create a new Version with specified generation and state
    ///
    /// 使用指定的代数和状态创建一个新 Version
    #[inline(always)]
    pub fn new(generation: Generation, state: u32) -> Self {
        debug_assert!(state <= 0b11);
        let g = generation.0.get();
        // Shift generation left by 2, add state
        // generation is NonZeroU32, so g >= 1. g << 2 >= 4.
        // So the result is always non-zero.
        let v = (g << 2) | (state & 0b11);
        unsafe { Self(NonZeroU32::new_unchecked(v)) }
    }

    /// Create a sentinel version
    ///
    /// 创建哨兵版本
    #[inline(always)]
    pub fn sentinel() -> Self {
        Self::new(Generation::MIN, 0b00)
    }

    /// Get the generation part
    ///
    /// 获取代数部分
    #[inline(always)]
    pub fn generation(&self) -> Generation {
        // Remove state bits and shift back
        unsafe { Generation::new_unchecked(self.0.get() >> 2) }
    }

    /// Get the state part (lowest 2 bits)
    ///
    /// 获取状态部分（最低 2 位）
    #[inline(always)]
    pub fn state(&self) -> u32 {
        self.0.get() & 0b11
    }

    /// Check if logic state is Vacant (0b00)
    ///
    /// 检查逻辑状态是否为空闲 (0b00)
    #[inline(always)]
    pub fn is_vacant(&self) -> bool {
        self.state() == 0b00
    }

    /// Check if logic state is Reserved (0b01)
    ///
    /// 检查逻辑状态是否为预留 (0b01)
    #[inline(always)]
    pub fn is_reserved(&self) -> bool {
        self.state() == 0b01
    }

    /// Check if logic state is Occupied (0b11)
    ///
    /// 检查逻辑状态是否被占用 (0b11)
    #[inline(always)]
    pub fn is_occupied(&self) -> bool {
        self.state() == 0b11
    }

    /// Transition: Vacant -> Reserved
    ///
    /// Increases value by 1 (0bXX00 -> 0bXX01)
    ///
    /// 状态转换：空闲 -> 预留
    ///
    /// 值增加 1 (0bXX00 -> 0bXX01)
    #[inline(always)]
    pub fn vacant_to_reserved(&mut self) {
        debug_assert!(self.is_vacant());
        // 0bXX00 + 1 = 0bXX01
        unsafe {
            self.0 = NonZeroU32::new_unchecked(self.0.get() + 1);
        }
    }

    /// Transition: Reserved -> Occupied
    ///
    /// Increases value by 2 (0bXX01 -> 0bXX11)
    ///
    /// 状态转换：预留 -> 占用
    ///
    /// 值增加 2 (0bXX01 -> 0bXX11)
    #[inline(always)]
    pub fn reserved_to_occupied(&mut self) {
        debug_assert!(self.is_reserved());
        // 0bXX01 + 2 = 0bXX11
        unsafe {
            self.0 = NonZeroU32::new_unchecked(self.0.get() + 2);
        }
    }

    /// Transition: Occupied -> Vacant (Next Generation)
    ///
    /// Increases value by 1 (0bXX11 -> 0bYY00), handles generation wrap
    ///
    /// 状态转换：占用 -> 空闲（下一代）
    ///
    /// 值增加 1 (0bXX11 -> 0bYY00)，处理代数回绕
    #[inline(always)]
    pub fn occupied_to_vacant(&mut self) {
        debug_assert!(self.is_occupied());
        let mut v = self.0.get();
        // 0bXX11 + 1 = 0bYY00 (where YY = XX + 1)
        v = v.wrapping_add(1);

        // If generation wraps to 0 (which means v >> 2 == 0), skip to 1
        if unlikely(v >> 2 == 0) {
            v = v.wrapping_add(1 << 2);
        }

        // Result is definitely non-zero because generation is at least 1 (shifted to 4)
        unsafe {
            self.0 = NonZeroU32::new_unchecked(v);
        }
    }

    /// Transition: Reserved -> Vacant (Next Generation)
    ///
    /// Used when releasing a handle.
    /// Increases value by 3 (0bXX01 -> 0bYY00), handles generation wrap
    ///
    /// 状态转换：预留 -> 空闲（下一代）
    ///
    /// 用于释放 handle 时。
    /// 值增加 3 (0bXX01 -> 0bYY00)，处理代数回绕
    #[inline(always)]
    pub fn reserved_to_vacant(&mut self) {
        debug_assert!(self.is_reserved());
        let mut v = self.0.get();
        // 0bXX01 + 3 = 0bYY00 (where YY = XX + 1)
        v = v.wrapping_add(3);

        // If generation wraps to 0, skip to 1
        if unlikely(v >> 2 == 0) {
            v = v.wrapping_add(1 << 2);
        }

        unsafe {
            self.0 = NonZeroU32::new_unchecked(v);
        }
    }
}

pub trait Key: Copy + Clone + PartialEq + Eq + Hash + Debug {
    fn from_parts(index: u32, generation: Generation, #[cfg(debug_assertions)] map_id: u64)
    -> Self;
    fn index(&self) -> u32;
    fn generation(&self) -> Generation;
    #[cfg(debug_assertions)]
    fn map_id(&self) -> u64;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DefaultKey {
    pub(crate) raw: u64,
    #[cfg(debug_assertions)]
    pub(crate) map_id: u64,
}

impl Key for DefaultKey {
    #[inline(always)]
    fn from_parts(
        index: u32,
        generation: Generation,
        #[cfg(debug_assertions)] map_id: u64,
    ) -> Self {
        Self {
            raw: ((generation.get() as u64) << 32) | (index as u64),
            #[cfg(debug_assertions)]
            map_id,
        }
    }

    #[inline(always)]
    fn index(&self) -> u32 {
        self.raw as u32
    }

    #[inline(always)]
    fn generation(&self) -> Generation {
        // We guarantee generation is non-zero upon creation
        unsafe { Generation(NonZeroU32::new_unchecked((self.raw >> 32) as u32)) }
    }

    #[cfg(debug_assertions)]
    #[inline(always)]
    fn map_id(&self) -> u64 {
        self.map_id
    }
}

impl DefaultKey {
    /// Create a new Key from index and generation
    ///
    /// 从 index 和 generation 创建新的 Key
    #[inline(always)]
    pub fn new(index: u32, generation: Generation, #[cfg(debug_assertions)] map_id: u64) -> Self {
        <Self as Key>::from_parts(
            index,
            generation,
            #[cfg(debug_assertions)]
            map_id,
        )
    }

    /// Decode Key into index and generation
    ///
    /// 将 Key 解码为 index 和 generation
    #[inline(always)]
    pub fn decode(&self) -> (u32, Generation) {
        (self.index(), self.generation())
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
