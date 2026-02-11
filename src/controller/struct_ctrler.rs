use std::sync::Mutex;

use bytemuck::Pod;

use crate::error::SMemResult;
use crate::smem::{self, SMemIF, SMemIFExt};
use crate::types::PAGE_SIZE;

/// Controller for a single struct value in shared memory.
///
/// Equivalent to C# `SMemCtrler<T>`. Provides read/write with caching.
pub struct SMemCtrler<T: Pod + Default + PartialEq> {
    smem: Option<Box<dyn SMemIF>>,
    cached: Mutex<T>,
}

impl<T: Pod + Default + PartialEq> SMemCtrler<T> {
    /// Create a new controller backed by platform shared memory.
    pub fn new(name: &str) -> SMemResult<Self> {
        let capacity = std::cmp::max(std::mem::size_of::<T>() as u64, PAGE_SIZE);
        let smem = smem::open_or_create(name, capacity)?;
        Ok(Self {
            smem: Some(smem),
            cached: Mutex::new(T::default()),
        })
    }

    /// Create a new controller using in-memory storage (no IPC).
    pub fn in_memory(name: &str) -> Self {
        let capacity = std::cmp::max(std::mem::size_of::<T>() as u64, PAGE_SIZE);
        Self {
            smem: Some(smem::open_in_memory(name, capacity)),
            cached: Mutex::new(T::default()),
        }
    }

    /// Create a controller with no backing store (cache only).
    pub fn no_smem() -> Self {
        Self {
            smem: None,
            cached: Mutex::new(T::default()),
        }
    }

    fn smem(&self) -> SMemResult<&dyn SMemIF> {
        self.smem
            .as_deref()
            .ok_or(crate::error::SMemError::NotInitialized)
    }

    /// Read the value from shared memory and update the cache.
    pub fn read(&self) -> SMemResult<T> {
        let value: T = self.smem()?.read_value(0)?;
        *self.cached.lock().unwrap() = value;
        Ok(value)
    }

    /// Write a value to shared memory and update the cache.
    pub fn write(&self, value: &T) -> SMemResult<()> {
        self.smem()?.write_value(0, value)?;
        *self.cached.lock().unwrap() = *value;
        Ok(())
    }

    /// Write a sub-field of the struct at a specific byte offset within the shared memory.
    pub fn write_field<F: Pod>(&self, offset: u64, field: &F) -> SMemResult<()> {
        self.smem()?.write_value(offset, field)?;
        // Re-read the full struct to update the cache
        if let Ok(value) = self.smem()?.read_value::<T>(0) {
            *self.cached.lock().unwrap() = value;
        }
        Ok(())
    }

    /// Try to read, returning `None` on error.
    pub fn try_read(&self) -> Option<T> {
        self.read().ok()
    }

    /// Try to write, returning `false` on error.
    pub fn try_write(&self, value: &T) -> bool {
        self.write(value).is_ok()
    }

    /// Get the cached value without reading from shared memory.
    pub fn cached_value(&self) -> T {
        *self.cached.lock().unwrap()
    }
}
