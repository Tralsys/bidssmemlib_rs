use bytemuck::{Pod, Zeroable};

/// Vehicle specification data.
///
/// Binary compatible with C# `Spec` struct (`LayoutKind.Sequential`).
/// Size: 20 bytes, Alignment: 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct Spec {
    /// Brake notch count
    pub b: i32,
    /// Power notch count
    pub p: i32,
    /// ATS check notch
    pub a: i32,
    /// Standard max brake notch (B67 equivalent)
    pub j: i32,
    /// Car count
    pub c: i32,
}
