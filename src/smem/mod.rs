pub mod in_memory;
#[cfg(windows)]
pub mod windows;

use bytemuck::Pod;

use crate::error::{SMemError, SMemResult};

/// Core shared memory interface (object-safe).
///
/// Provides low-level byte-level read/write access to a named shared memory region.
/// Implementations must handle internal synchronization (reader-writer locking).
pub trait SMemIF: Send + Sync {
    /// Returns the name of the shared memory region.
    fn name(&self) -> &str;

    /// Returns the capacity in bytes.
    fn capacity(&self) -> u64;

    /// Returns true if this instance created the shared memory (vs. opening existing).
    fn is_newly_created(&self) -> bool;

    /// Read bytes from the shared memory at the given offset.
    fn read_bytes(&self, pos: u64, buf: &mut [u8]) -> SMemResult<()>;

    /// Write bytes to the shared memory at the given offset.
    fn write_bytes(&self, pos: u64, data: &[u8]) -> SMemResult<()>;
}

/// Extension trait that adds generic (typed) read/write on top of the byte-level `SMemIF`.
///
/// Automatically implemented for all `SMemIF` implementors.
pub trait SMemIFExt: SMemIF {
    /// Read a `Pod` type from shared memory at the given byte offset.
    fn read_value<T: Pod>(&self, pos: u64) -> SMemResult<T> {
        let size = std::mem::size_of::<T>() as u64;
        if size == 0 {
            return Ok(bytemuck::Zeroable::zeroed());
        }
        let mut value: T = bytemuck::Zeroable::zeroed();
        self.read_bytes(pos, bytemuck::bytes_of_mut(&mut value))?;
        Ok(value)
    }

    /// Write a `Pod` type to shared memory at the given byte offset.
    fn write_value<T: Pod>(&self, pos: u64, value: &T) -> SMemResult<()> {
        self.write_bytes(pos, bytemuck::bytes_of(value))
    }

    /// Read an array of `Pod` elements from shared memory.
    fn read_array<T: Pod>(&self, pos: u64, count: usize) -> SMemResult<Vec<T>> {
        if count == 0 {
            return Ok(Vec::new());
        }
        let mut buf: Vec<T> = vec![bytemuck::Zeroable::zeroed(); count];
        self.read_bytes(pos, bytemuck::cast_slice_mut(&mut buf))?;
        Ok(buf)
    }

    /// Write an array of `Pod` elements to shared memory.
    fn write_array<T: Pod>(&self, pos: u64, values: &[T]) -> SMemResult<()> {
        if values.is_empty() {
            return Ok(());
        }
        self.write_bytes(pos, bytemuck::cast_slice(values))
    }
}

impl<S: SMemIF + ?Sized> SMemIFExt for S {}

/// Create or open a shared memory region with the platform-appropriate backend.
///
/// On Windows, this creates a named memory-mapped file (Win32 API).
/// On non-Windows platforms, this uses an in-memory fallback.
pub fn open_or_create(name: &str, capacity: u64) -> SMemResult<Box<dyn SMemIF>> {
    #[cfg(windows)]
    {
        let backend = windows::WindowsSMemIF::open_or_create(name, capacity)?;
        Ok(Box::new(backend))
    }
    #[cfg(not(windows))]
    {
        let backend = in_memory::InMemorySMemIF::new(name, capacity);
        Ok(Box::new(backend))
    }
}

/// Create an in-memory (non-IPC) shared memory region. Available on all platforms.
pub fn open_in_memory(name: &str, capacity: u64) -> Box<dyn SMemIF> {
    Box::new(in_memory::InMemorySMemIF::new(name, capacity))
}

/// Validate that a read/write at `pos` with `size` bytes fits within `capacity`.
pub(crate) fn validate_bounds(pos: u64, size: u64, capacity: u64) -> SMemResult<()> {
    if pos.checked_add(size).is_none_or(|end| end > capacity) {
        return Err(SMemError::OutOfBounds {
            pos,
            size,
            capacity,
        });
    }
    Ok(())
}
