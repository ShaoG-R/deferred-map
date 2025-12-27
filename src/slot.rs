use std::fmt;
use std::mem::ManuallyDrop;

/// Internal slot storage using union for memory optimization
///
/// When occupied, stores the value; when vacant, stores the next free slot index.
///
/// Slot 内部存储，使用 union 优化内存
///
/// 占用时存储值，空闲时存储下一个空闲 slot 的索引
pub(crate) union SlotUnion<T> {
    pub(crate) value: ManuallyDrop<T>,
    pub(crate) next_free: u32,
}

/// Slot stores the actual value and version information
///
/// Version uses lowest 2 bits for state (0b00=vacant, 0b01=reserved, 0b11=occupied)
/// and upper 30 bits for generation counter.
///
/// Slot 存储实际的值和 version 信息
///
/// version 的最低 2 位表示状态（0b00=空闲, 0b01=预留, 0b11=占用）
/// 高 30 位为代数计数器
pub(crate) struct Slot<T> {
    pub(crate) u: SlotUnion<T>,
    pub(crate) version: u32, // Low 2 bits: state, High 30 bits: generation | 低2位：状态，高30位：代数
}

/// Safe API to read slot content
///
/// 安全的 API 来读取 slot
pub(crate) enum SlotContent<'a, T: 'a> {
    /// Occupied slot with value
    ///
    /// 占用 slot 的值
    Occupied(&'a T),

    /// Vacant slot with next free index
    ///
    /// 空闲 slot 的索引
    Vacant(&'a u32),
}

/// Safe API to read mutable slot content
///
/// 安全的 API 来读取可变 slot
pub(crate) enum SlotContentMut<'a, T: 'a> {
    /// Occupied slot with mutable reference
    ///
    /// 占用 slot 的可变引用
    OccupiedMut(&'a mut T),

    /// Vacant slot with mutable reference to next free index
    ///
    /// 空闲 slot 的索引可变引用
    VacantMut(&'a mut u32),
}

use self::SlotContent::{Occupied, Vacant};
use self::SlotContentMut::{OccupiedMut, VacantMut};

impl<T> Slot<T> {
    /// Get the state bits (lowest 2 bits of version)
    ///
    /// 获取状态位（version 的最低 2 位）
    #[inline(always)]
    fn state_bits(&self) -> u32 {
        self.version & 0b11
    }

    /// Check if slot is vacant (state bits == 0b00)
    ///
    /// 检查 slot 是否空闲（状态位 == 0b00）
    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn is_vacant(&self) -> bool {
        self.state_bits() == 0b00
    }

    /// Check if slot is reserved (state bits == 0b01)
    ///
    /// 检查 slot 是否已预留（状态位 == 0b01）
    #[inline(always)]
    pub(crate) fn is_reserved(&self) -> bool {
        self.state_bits() == 0b01
    }

    /// Check if slot is occupied (state bits == 0b11)
    ///
    /// 检查 slot 是否被占用（状态位 == 0b11）
    #[inline(always)]
    pub(crate) fn is_occupied(&self) -> bool {
        self.state_bits() == 0b11
    }

    /// Get the generation from version (excludes state bits)
    ///
    /// 从 version 中获取 generation（不包含状态位）
    #[inline(always)]
    pub(crate) fn generation(&self) -> u32 {
        self.version >> 2
    }

    /// Safely get slot content
    ///
    /// Only returns content for Vacant (next_free) or Occupied (value) states.
    /// Reserved state is treated as vacant for safety.
    ///
    /// 安全地获取 slot 内容
    ///
    /// 只返回 Vacant (next_free) 或 Occupied (value) 状态的内容
    /// Reserved 状态出于安全考虑视为 vacant
    #[inline(always)]
    pub(crate) fn get<'a>(&'a self) -> SlotContent<'a, T> {
        unsafe {
            if self.is_occupied() {
                Occupied(&*self.u.value)
            } else {
                // For vacant or reserved, return next_free (for reserved it's undefined but safe to read)
                // 对于 vacant 或 reserved，返回 next_free（对于 reserved 它是未定义的但安全读取）
                Vacant(&self.u.next_free)
            }
        }
    }

    /// Safely get mutable slot content
    ///
    /// Only returns content for Vacant (next_free) or Occupied (value) states.
    /// Reserved state is treated as vacant for safety.
    ///
    /// 安全地获取 slot 可变内容
    ///
    /// 只返回 Vacant (next_free) 或 Occupied (value) 状态的内容
    /// Reserved 状态出于安全考虑视为 vacant
    #[inline(always)]
    pub(crate) fn get_mut<'a>(&'a mut self) -> SlotContentMut<'a, T> {
        unsafe {
            if self.is_occupied() {
                OccupiedMut(&mut *self.u.value)
            } else {
                // For vacant or reserved, return next_free (for reserved it's undefined but safe to read)
                // 对于 vacant 或 reserved，返回 next_free（对于 reserved 它是未定义的但安全读取）
                VacantMut(&mut self.u.next_free)
            }
        }
    }
}

impl<T> Drop for Slot<T> {
    #[inline]
    fn drop(&mut self) {
        if std::mem::needs_drop::<T>() && self.is_occupied() {
            // Only drop value when occupied
            // 只有在占用状态时才 drop 值
            unsafe {
                ManuallyDrop::drop(&mut self.u.value);
            }
        }
    }
}

impl<T: Clone> Clone for Slot<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            u: match self.get() {
                Occupied(value) => SlotUnion {
                    value: ManuallyDrop::new(value.clone()),
                },
                Vacant(&next_free) => SlotUnion { next_free },
            },
            version: self.version,
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self.get_mut(), source.get()) {
            (OccupiedMut(self_val), Occupied(source_val)) => self_val.clone_from(source_val),
            (VacantMut(self_next_free), Vacant(&source_next_free)) => {
                *self_next_free = source_next_free
            }
            (_, Occupied(value)) => {
                self.u = SlotUnion {
                    value: ManuallyDrop::new(value.clone()),
                }
            }
            (_, Vacant(&next_free)) => self.u = SlotUnion { next_free },
        }
        self.version = source.version;
    }
}

impl<T: fmt::Debug> fmt::Debug for Slot<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = fmt.debug_struct("Slot");
        builder.field("version", &self.version);
        match self.get() {
            Occupied(value) => builder.field("value", value).finish(),
            Vacant(next_free) => builder.field("next_free", next_free).finish(),
        }
    }
}

#[cfg(feature = "serde")]
impl<T: serde::Serialize> serde::Serialize for Slot<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Slot", 2)?;
        state.serialize_field("version", &self.version)?;

        #[derive(serde::Serialize)]
        enum SlotInner<'a, T> {
            Occupied(&'a T),
            Vacant(u32),
            Reserved,
        }

        let inner = if self.is_occupied() {
            unsafe { SlotInner::Occupied(&*self.u.value) }
        } else if self.is_reserved() {
            SlotInner::Reserved
        } else {
            unsafe { SlotInner::Vacant(self.u.next_free) }
        };

        state.serialize_field("inner", &inner)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de, T: serde::Deserialize<'de>> serde::Deserialize<'de> for Slot<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct SlotHelper<T> {
            version: u32,
            inner: SlotInnerOwned<T>,
        }

        #[derive(serde::Deserialize)]
        enum SlotInnerOwned<T> {
            Occupied(T),
            Vacant(u32),
            Reserved,
        }

        let helper = SlotHelper::<T>::deserialize(deserializer)?;

        let state_bits = helper.version & 0b11;
        match (&helper.inner, state_bits) {
            (SlotInnerOwned::Occupied(_), 0b11) => {}
            (SlotInnerOwned::Vacant(_), 0b00) => {}
            (SlotInnerOwned::Reserved, 0b01) => {}
            _ => {
                return Err(serde::de::Error::custom(
                    "Slot version and content mismatch",
                ));
            }
        }

        let u = match helper.inner {
            SlotInnerOwned::Occupied(val) => SlotUnion {
                value: ManuallyDrop::new(val),
            },
            SlotInnerOwned::Vacant(next) => SlotUnion { next_free: next },
            SlotInnerOwned::Reserved => SlotUnion { next_free: 0 },
        };

        Ok(Slot {
            u,
            version: helper.version,
        })
    }
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn test_slot_occupied_serde() {
        let slot = Slot {
            u: SlotUnion {
                value: ManuallyDrop::new(42),
            },
            version: 0b11 | (1 << 2), // Occupied, generation 1
        };

        let serialized = serde_json::to_string(&slot).expect("Failed to serialize");
        let deserialized: Slot<i32> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert!(deserialized.is_occupied());
        assert_eq!(deserialized.generation(), 1);
        if let Occupied(val) = deserialized.get() {
            assert_eq!(*val, 42);
        } else {
            panic!("Expected Occupied");
        }
    }

    #[test]
    fn test_slot_vacant_serde() {
        let slot: Slot<i32> = Slot {
            u: SlotUnion { next_free: 10 },
            version: 0b00 | (2 << 2), // Vacant, generation 2
        };

        let serialized = serde_json::to_string(&slot).expect("Failed to serialize");
        let deserialized: Slot<i32> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert!(deserialized.is_vacant());
        assert_eq!(deserialized.generation(), 2);
        if let Vacant(next) = deserialized.get() {
            assert_eq!(*next, 10);
        } else {
            panic!("Expected Vacant");
        }
    }

    #[test]
    fn test_slot_reserved_serde() {
        let slot: Slot<i32> = Slot {
            u: SlotUnion { next_free: 0 },
            version: 0b01 | (3 << 2), // Reserved, generation 3
        };

        let serialized = serde_json::to_string(&slot).expect("Failed to serialize");
        let deserialized: Slot<i32> =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert!(deserialized.is_reserved());
        assert_eq!(deserialized.generation(), 3);
    }
}
