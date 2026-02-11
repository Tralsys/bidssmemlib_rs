use bytemuck::{Pod, Zeroable};

/// Vehicle state data.
///
/// Binary compatible with C# `State` struct (`LayoutKind.Sequential`).
/// Size: 40 bytes, Alignment: 8.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable, Default)]
#[repr(C)]
pub struct State {
    /// Location [m]
    pub z: f64,
    /// Speed [km/h]
    pub v: f32,
    /// Time [ms from midnight]
    pub t: i32,
    /// BC pressure [kPa]
    pub bc: f32,
    /// MR pressure [kPa]
    pub mr: f32,
    /// ER pressure [kPa]
    pub er: f32,
    /// BP pressure [kPa]
    pub bp: f32,
    /// SAP pressure [kPa]
    pub sap: f32,
    /// Current [A]
    pub i: f32,
}
