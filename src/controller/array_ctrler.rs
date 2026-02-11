use std::sync::Mutex;

use bytemuck::Pod;

use crate::error::{SMemError, SMemResult};
use crate::smem::{self, SMemIF, SMemIFExt};
use crate::types::PAGE_SIZE;

/// Controller for array data in shared memory.
///
/// Equivalent to C# `ArrayDataSMemCtrler<T>`.
///
/// Two-tier storage strategy (matching C# behavior):
/// - **Small arrays**: `[i32 length][T; elements...]` in a single MMF.
///   Max inline count: `(PAGE_SIZE - 4) / sizeof(T)`
/// - **Large arrays**: length stored in `"{name}_length"` MMF (as i32),
///   data in `"{name}_{count}"` MMF.
pub struct ArrayDataSMemCtrler<T: Pod + Default + PartialEq> {
    name: String,
    use_smem: bool,
    /// Primary shared memory (for small arrays: [i32 length, T...])
    smem: Option<Box<dyn SMemIF>>,
    /// Separate MMF for large array data; recreated when length changes
    smem_array: Mutex<Option<Box<dyn SMemIF>>>,
    /// Current known large-array element count
    current_large_count: Mutex<usize>,
    /// Cached array data
    cached: Mutex<Vec<T>>,
    /// Maximum elements that fit in the small (inline) tier
    elem_max_count: usize,
}

impl<T: Pod + Default + PartialEq> ArrayDataSMemCtrler<T> {
    /// Create a new controller backed by platform shared memory.
    pub fn new(name: &str) -> SMemResult<Self> {
        let elem_size = std::mem::size_of::<T>();
        let elem_max_count = if elem_size > 0 {
            (PAGE_SIZE as usize - std::mem::size_of::<i32>()) / elem_size
        } else {
            0
        };

        let smem = smem::open_or_create(name, PAGE_SIZE)?;

        Ok(Self {
            name: name.to_string(),
            use_smem: true,
            smem: Some(smem),
            smem_array: Mutex::new(None),
            current_large_count: Mutex::new(0),
            cached: Mutex::new(Vec::new()),
            elem_max_count,
        })
    }

    /// Create a new controller using in-memory storage (no IPC).
    pub fn in_memory(name: &str) -> Self {
        let elem_size = std::mem::size_of::<T>();
        let elem_max_count = if elem_size > 0 {
            (PAGE_SIZE as usize - std::mem::size_of::<i32>()) / elem_size
        } else {
            0
        };

        Self {
            name: name.to_string(),
            use_smem: true,
            smem: Some(smem::open_in_memory(name, PAGE_SIZE)),
            smem_array: Mutex::new(None),
            current_large_count: Mutex::new(0),
            cached: Mutex::new(Vec::new()),
            elem_max_count,
        }
    }

    fn smem(&self) -> SMemResult<&dyn SMemIF> {
        self.smem.as_deref().ok_or(SMemError::NotInitialized)
    }

    fn open_smem(&self, name: &str, capacity: u64) -> SMemResult<Box<dyn SMemIF>> {
        if self.use_smem {
            smem::open_or_create(name, capacity)
        } else {
            Ok(smem::open_in_memory(name, capacity))
        }
    }

    /// Ensure the large-array MMF exists with the right size.
    fn ensure_large_array_smem(&self, count: usize) -> SMemResult<()> {
        let mut current = self.current_large_count.lock().unwrap();
        if *current == count && self.smem_array.lock().unwrap().is_some() {
            return Ok(());
        }

        let elem_size = std::mem::size_of::<T>();
        let capacity = (count * elem_size) as u64;
        let array_name = format!("{}_{}", self.name, count);
        let new_smem = self.open_smem(&array_name, capacity)?;

        *self.smem_array.lock().unwrap() = Some(new_smem);
        *current = count;
        Ok(())
    }

    /// Read the entire array from shared memory.
    pub fn read(&self) -> SMemResult<Vec<T>> {
        let smem = self.smem()?;
        // Read the length prefix
        let length: i32 = smem.read_value(0)?;
        let count = length.max(0) as usize;

        if count == 0 {
            let mut cached = self.cached.lock().unwrap();
            *cached = Vec::new();
            return Ok(Vec::new());
        }

        let values = if count <= self.elem_max_count {
            // Small tier: data follows the i32 length prefix
            let offset = std::mem::size_of::<i32>() as u64;
            smem.read_array(offset, count)?
        } else {
            // Large tier: read from separate array MMF
            self.ensure_large_array_smem(count)?;
            let array_lock = self.smem_array.lock().unwrap();
            let array_smem = array_lock.as_deref().ok_or(SMemError::NotInitialized)?;
            array_smem.read_array(0, count)?
        };

        let mut cached = self.cached.lock().unwrap();
        *cached = values.clone();
        Ok(values)
    }

    /// Write the entire array to shared memory.
    pub fn write(&self, values: &[T]) -> SMemResult<()> {
        let smem = self.smem()?;
        let count = values.len();
        let length = count as i32;

        // Write the length prefix
        smem.write_value(0, &length)?;

        if count == 0 {
            let mut cached = self.cached.lock().unwrap();
            *cached = Vec::new();
            return Ok(());
        }

        if count <= self.elem_max_count {
            // Small tier
            let offset = std::mem::size_of::<i32>() as u64;
            smem.write_array(offset, values)?;
        } else {
            // Large tier
            self.ensure_large_array_smem(count)?;
            let array_lock = self.smem_array.lock().unwrap();
            let array_smem = array_lock.as_deref().ok_or(SMemError::NotInitialized)?;
            array_smem.write_array(0, values)?;
        }

        let mut cached = self.cached.lock().unwrap();
        *cached = values.to_vec();
        Ok(())
    }

    /// Read a single element by index.
    pub fn read_at(&self, index: usize) -> SMemResult<T> {
        let smem = self.smem()?;
        let length: i32 = smem.read_value(0)?;
        let count = length.max(0) as usize;

        if index >= count {
            return Err(SMemError::IndexOutOfRange {
                index,
                length: count,
            });
        }

        let elem_size = std::mem::size_of::<T>() as u64;

        if count <= self.elem_max_count {
            let offset = std::mem::size_of::<i32>() as u64 + index as u64 * elem_size;
            smem.read_value(offset)
        } else {
            self.ensure_large_array_smem(count)?;
            let array_lock = self.smem_array.lock().unwrap();
            let array_smem = array_lock.as_deref().ok_or(SMemError::NotInitialized)?;
            array_smem.read_value(index as u64 * elem_size)
        }
    }

    /// Write a single element by index (the array must have been written with sufficient length first).
    pub fn write_at(&self, index: usize, value: &T) -> SMemResult<()> {
        let smem = self.smem()?;
        let length: i32 = smem.read_value(0)?;
        let count = length.max(0) as usize;

        if index >= count {
            return Err(SMemError::IndexOutOfRange {
                index,
                length: count,
            });
        }

        let elem_size = std::mem::size_of::<T>() as u64;

        if count <= self.elem_max_count {
            let offset = std::mem::size_of::<i32>() as u64 + index as u64 * elem_size;
            smem.write_value(offset, value)
        } else {
            self.ensure_large_array_smem(count)?;
            let array_lock = self.smem_array.lock().unwrap();
            let array_smem = array_lock.as_deref().ok_or(SMemError::NotInitialized)?;
            array_smem.write_value(index as u64 * elem_size, value)
        }
    }

    /// Get the cached array without reading from shared memory.
    pub fn cached_value(&self) -> Vec<T> {
        self.cached.lock().unwrap().clone()
    }
}
