use bytemuck::{Pod, Zeroable};

/// Preceding train data (part of OpenD).
///
/// Binary compatible with C# `OpenD.PreTrainD` (`LayoutKind.Sequential`).
/// Size: 32 bytes, Alignment: 8.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct PreTrainD {
    is_enabled: u8,
    _pad0: [u8; 7],
    /// Preceding train tail position [m]
    pub location: f64,
    /// Distance to preceding train [m]
    pub distance: f64,
    /// Preceding train speed [km/h]
    pub speed: f64,
}

impl Default for PreTrainD {
    fn default() -> Self {
        Zeroable::zeroed()
    }
}

impl PreTrainD {
    pub fn is_enabled(&self) -> bool {
        self.is_enabled != 0
    }

    pub fn set_is_enabled(&mut self, val: bool) {
        self.is_enabled = val as u8;
    }
}

/// OpenBVE extended data.
///
/// Binary compatible with C# `OpenD` struct (`LayoutKind.Sequential`).
/// Size: 80 bytes, Alignment: 8.
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct OpenD {
    is_enabled: u8,
    _pad0: [u8; 3],
    /// Structure version
    pub ver: i32,
    /// Curve radius [m]
    pub radius: f64,
    /// Cant magnitude [mm]
    pub cant: f64,
    /// Gradient [permil]
    pub pitch: f64,
    /// Frame elapsed time [ms]
    pub elap_time: f64,
    /// Preceding train data
    pub pre_train: PreTrainD,
    /// Self-brake notch count
    pub self_b_count: i32,
    /// Self-brake handle position
    pub self_b_position: i32,
}

impl Default for OpenD {
    fn default() -> Self {
        Zeroable::zeroed()
    }
}

impl OpenD {
    pub fn is_enabled(&self) -> bool {
        self.is_enabled != 0
    }

    pub fn set_is_enabled(&mut self, val: bool) {
        self.is_enabled = val as u8;
    }
}
