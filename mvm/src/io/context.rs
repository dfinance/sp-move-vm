use alloc::vec::Vec;
use move_core_types::language_storage::StructTag;

pub const TIMESTAMP_MODULE: &str = "DiemTimestamp";
pub const CURRENT_TIME_MICROSECONDS: &str = "CurrentTimeMicroseconds";

pub const BLOCK_MODULE: &str = "DiemBlock";
pub const BLOCK_METADATA: &str = "BlockMetadata";

#[derive(Debug)]
pub struct ExecutionContext {
    pub timestamp: u64,
    pub block_height: u64,
}

impl ExecutionContext {
    pub fn new(timestamp: u64, block_height: u64) -> ExecutionContext {
        ExecutionContext {
            timestamp,
            block_height,
        }
    }

    pub fn resolve(&self, tag: &StructTag) -> Option<Vec<u8>> {
        if tag.module.as_str() == TIMESTAMP_MODULE && tag.name.as_str() == CURRENT_TIME_MICROSECONDS
        {
            bcs::to_bytes(&self.timestamp).ok()
        } else if tag.module.as_str() == BLOCK_MODULE && tag.name.as_str() == BLOCK_METADATA {
            bcs::to_bytes(&self.block_height).ok()
        } else {
            None
        }
    }
}
