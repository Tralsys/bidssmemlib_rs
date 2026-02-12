use bytemuck::{Pod, Zeroable};

/// Handle position data.
///
/// Binary compatible with C# `Hand` struct (`LayoutKind.Sequential`).
/// Size: 16 bytes, Alignment: 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct Hand {
    /// Brake notch position
    pub b: i32,
    /// Power notch position
    pub p: i32,
    /// Reverser position (-1: backward, 0: neutral, 1: forward)
    pub r: i32,
    /// Constant speed control (0: continue, 1: enable, 2: disable)
    pub c: i32,
}
