/// Branch prediction hint: marks code path as unlikely (cold)
/// 
/// This function is used to hint the compiler that a branch is unlikely to be taken,
/// which can improve performance by optimizing the common path.
/// 
/// 分支预测提示：标记代码路径为不太可能执行（冷路径）
/// 
/// 此函数用于提示编译器某个分支不太可能被执行，
/// 通过优化常见路径来提高性能。
#[inline(always)]
#[cold]
pub(crate) fn cold() {}

/// Branch prediction hint: likely condition
/// 
/// Hints to the compiler that the condition is likely to be true.
/// Returns the original boolean value.
/// 
/// 分支预测提示：提示编译器这个条件很可能为真
/// 
/// # Parameters
/// - `b`: The boolean condition to evaluate
/// 
/// # Returns
/// The original boolean value
/// 
/// # 参数
/// - `b`: 要评估的布尔条件
/// 
/// # 返回值
/// 原始的布尔值
#[inline(always)]
pub(crate) fn likely(b: bool) -> bool {
    if !b {
        cold();
    }
    b
}

/// Branch prediction hint: unlikely condition
/// 
/// Hints to the compiler that the condition is unlikely to be true.
/// Returns the original boolean value.
/// 
/// 分支预测提示：提示编译器这个条件很可能为假
/// 
/// # Parameters
/// - `b`: The boolean condition to evaluate
/// 
/// # Returns
/// The original boolean value
/// 
/// # 参数
/// - `b`: 要评估的布尔条件
/// 
/// # 返回值
/// 原始的布尔值
#[inline(always)]
pub(crate) fn unlikely(b: bool) -> bool {
    if b {
        cold();
    }
    b
}

