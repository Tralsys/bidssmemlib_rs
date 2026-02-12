use std::sync::RwLock;

use crate::error::{SMemError, SMemResult};
use crate::smem::{validate_bounds, SMemIF};

/// In-memory shared memory implementation.
///
/// Does not provide inter-process communication — all data stays within
/// the current process. Used as a fallback on non-Windows platforms and
/// for testing.
pub struct InMemorySMemIF {
    name: String,
    data: RwLock<Vec<u8>>,
}

impl InMemorySMemIF {
    /// Create a new in-memory shared memory region with the given name and capacity.
    pub fn new(name: &str, capacity: u64) -> Self {
        Self {
            name: name.to_string(),
            data: RwLock::new(vec![0u8; capacity as usize]),
        }
    }
}

impl SMemIF for InMemorySMemIF {
    fn name(&self) -> &str {
        &self.name
    }

    fn capacity(&self) -> u64 {
        self.data.read().unwrap().len() as u64
    }

    fn is_newly_created(&self) -> bool {
        true
    }

    fn read_bytes(&self, pos: u64, buf: &mut [u8]) -> SMemResult<()> {
        let size = buf.len() as u64;
        let data = self
            .data
            .read()
            .map_err(|e| SMemError::Platform(format!("RwLock poisoned: {e}")))?;
        validate_bounds(pos, size, data.len() as u64)?;
        let start = pos as usize;
        buf.copy_from_slice(&data[start..start + buf.len()]);
        Ok(())
    }

    fn write_bytes(&self, pos: u64, data: &[u8]) -> SMemResult<()> {
        let size = data.len() as u64;
        let mut storage = self
            .data
            .write()
            .map_err(|e| SMemError::Platform(format!("RwLock poisoned: {e}")))?;
        validate_bounds(pos, size, storage.len() as u64)?;
        let start = pos as usize;
        storage[start..start + data.len()].copy_from_slice(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_read_write() {
        let smem = InMemorySMemIF::new("test", 64);
        let data = [1u8, 2, 3, 4];
        smem.write_bytes(0, &data).unwrap();

        let mut buf = [0u8; 4];
        smem.read_bytes(0, &mut buf).unwrap();
        assert_eq!(buf, data);
    }

    #[test]
    fn test_out_of_bounds() {
        let smem = InMemorySMemIF::new("test", 4);
        let result = smem.write_bytes(2, &[1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_name_and_capacity() {
        let smem = InMemorySMemIF::new("hello", 128);
        assert_eq!(smem.name(), "hello");
        assert_eq!(smem.capacity(), 128);
        assert!(smem.is_newly_created());
    }
}
