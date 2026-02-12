use bytemuck::{Pod, Zeroable};

use super::{Hand, Spec, State};

/// Main BIDS shared memory data structure.
///
/// Binary compatible with C# `BIDSSharedMemoryData` (`LayoutKind.Sequential`).
/// Size: 96 bytes, Alignment: 8.
///
/// Bool fields are stored as `u8` (0 or 1) for `bytemuck::Pod` compatibility.
/// Use the accessor methods `is_enabled()` / `set_is_enabled()` etc.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct BIDSSharedMemoryData {
    is_enabled: u8,
    _pad0: [u8; 3],
    pub version_num: i32,
    pub spec_data: Spec,
    _pad1: [u8; 4],
    pub state_data: State,
    pub handle_data: Hand,
    is_door_closed: u8,
    _pad2: [u8; 7],
}

impl Default for BIDSSharedMemoryData {
    fn default() -> Self {
        Zeroable::zeroed()
    }
}

impl BIDSSharedMemoryData {
    /// Whether the shared memory data is valid/enabled.
    pub fn is_enabled(&self) -> bool {
        self.is_enabled != 0
    }

    /// Set whether the shared memory data is valid/enabled.
    pub fn set_is_enabled(&mut self, val: bool) {
        self.is_enabled = val as u8;
    }

    /// Whether all doors are closed.
    pub fn is_door_closed(&self) -> bool {
        self.is_door_closed != 0
    }

    /// Set whether all doors are closed.
    pub fn set_is_door_closed(&mut self, val: bool) {
        self.is_door_closed = val as u8;
    }
}
