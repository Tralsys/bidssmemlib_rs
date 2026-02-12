use std::ptr;
use std::sync::RwLock;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::System::Memory::{
    CreateFileMappingW, MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_ALL_ACCESS,
    MEMORY_MAPPED_VIEW_ADDRESS, PAGE_READWRITE,
};

use crate::error::{SMemError, SMemResult};
use crate::smem::{validate_bounds, SMemIF};
use crate::types::PAGE_SIZE;

/// Windows shared memory implementation using Win32 Memory-Mapped Files.
///
/// Uses `CreateFileMappingW` / `OpenFileMappingW` / `MapViewOfFile` to create
/// or open named shared memory backed by the system paging file.
///
/// Binary compatible with C#'s `MemoryMappedFile.CreateOrOpen`.
pub struct WindowsSMemIF {
    name: String,
    capacity: u64,
    is_newly_created: bool,
    handle: HANDLE,
    view: MEMORY_MAPPED_VIEW_ADDRESS,
    lock: RwLock<()>,
}

// SAFETY: The raw pointer in `view` is valid for the lifetime of the mapping handle.
// Concurrent access is protected by the internal RwLock.
unsafe impl Send for WindowsSMemIF {}
unsafe impl Sync for WindowsSMemIF {}

impl WindowsSMemIF {
    /// Open an existing or create a new named shared memory region.
    ///
    /// Capacity is rounded up to the nearest multiple of PAGE_SIZE (4096).
    pub fn open_or_create(name: &str, capacity: u64) -> SMemResult<Self> {
        let capacity = round_up_to_page(capacity);
        let wide_name = to_wide_string(name);
        let pcwstr = PCWSTR(wide_name.as_ptr());

        // Try to open existing first
        let (handle, is_newly_created) = unsafe {
            let existing = OpenFileMappingW(FILE_MAP_ALL_ACCESS.0, false, pcwstr);
            match existing {
                Ok(h) => (h, false),
                Err(_) => {
                    // Create new
                    let high = (capacity >> 32) as u32;
                    let low = capacity as u32;
                    let h = CreateFileMappingW(
                        HANDLE::default(), // system paging file
                        None,              // default security
                        PAGE_READWRITE,
                        high,
                        low,
                        pcwstr,
                    )
                    .map_err(|e| SMemError::Platform(format!("CreateFileMappingW failed: {e}")))?;
                    (h, true)
                }
            }
        };

        let view = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, 0) };

        if view.Value.is_null() {
            unsafe {
                let _ = CloseHandle(handle);
            }
            return Err(SMemError::Platform("MapViewOfFile returned null".into()));
        }

        Ok(Self {
            name: name.to_string(),
            capacity,
            is_newly_created,
            handle,
            view,
            lock: RwLock::new(()),
        })
    }
}

impl SMemIF for WindowsSMemIF {
    fn name(&self) -> &str {
        &self.name
    }

    fn capacity(&self) -> u64 {
        self.capacity
    }

    fn is_newly_created(&self) -> bool {
        self.is_newly_created
    }

    fn read_bytes(&self, pos: u64, buf: &mut [u8]) -> SMemResult<()> {
        validate_bounds(pos, buf.len() as u64, self.capacity)?;
        let _guard = self
            .lock
            .read()
            .map_err(|e| SMemError::Platform(format!("RwLock poisoned: {e}")))?;
        unsafe {
            let src = (self.view.Value as *const u8).add(pos as usize);
            ptr::copy_nonoverlapping(src, buf.as_mut_ptr(), buf.len());
        }
        Ok(())
    }

    fn write_bytes(&self, pos: u64, data: &[u8]) -> SMemResult<()> {
        validate_bounds(pos, data.len() as u64, self.capacity)?;
        let _guard = self
            .lock
            .write()
            .map_err(|e| SMemError::Platform(format!("RwLock poisoned: {e}")))?;
        unsafe {
            let dst = (self.view.Value as *mut u8).add(pos as usize);
            ptr::copy_nonoverlapping(data.as_ptr(), dst, data.len());
        }
        Ok(())
    }
}

impl Drop for WindowsSMemIF {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(self.view);
            let _ = CloseHandle(self.handle);
        }
    }
}

fn round_up_to_page(size: u64) -> u64 {
    let page = PAGE_SIZE;
    if size == 0 {
        return page;
    }
    (size + page - 1) / page * page
}

fn to_wide_string(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
