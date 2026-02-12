use bytemuck::{Pod, Zeroable};

/// Extended handle data for input control.
///
/// Binary compatible with C# `Hands` struct (`LayoutKind.Sequential`).
/// Size: 32 bytes, Alignment: 8.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct Hands {
    /// Brake handle position (notch)
    pub b: i32,
    /// Power handle position (notch)
    pub p: i32,
    /// Reverser handle position (-1, 0, 1)
    pub r: i32,
    /// Self-brake handle position
    pub s: i32,
    /// Brake position (0.0 ~ 1.0 analog)
    pub b_pos: f64,
    /// Power position (0.0 ~ 1.0 analog)
    pub p_pos: f64,
}
