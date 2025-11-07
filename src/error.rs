use std::fmt;

/// Error type for DeferredMap operations
/// 
/// DeferredMap 操作的错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeferredMapError {
    /// Handle has already been used
    /// 
    /// Handle 已被使用
    HandleAlreadyUsed,
    
    /// Invalid handle
    /// 
    /// 无效的 Handle
    InvalidHandle,
    
    /// Generation mismatch
    /// 
    /// Generation 不匹配
    GenerationMismatch,
}

impl fmt::Display for DeferredMapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeferredMapError::HandleAlreadyUsed => write!(f, "Handle has already been used"),
            DeferredMapError::InvalidHandle => write!(f, "Invalid handle"),
            DeferredMapError::GenerationMismatch => write!(f, "Generation mismatch"),
        }
    }
}

impl std::error::Error for DeferredMapError {}

