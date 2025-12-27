mod handle;
mod map;
mod slot;
mod utils;

pub use handle::Handle;
pub use map::DeferredMap;

#[cfg(test)]
mod tests {
    // Test modules for DeferredMap
    // DeferredMap 的测试模块
    mod edge_cases;
    mod handle;
    mod insertion;
    mod removal;
}
