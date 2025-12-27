mod handle;
mod map;
mod secondary;
mod slot;
mod utils;

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
