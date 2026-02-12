mod bsmd;
mod hand;
mod hands;
mod open_d;
mod spec;
mod state;

pub use bsmd::BIDSSharedMemoryData;
pub use hand::Hand;
pub use hands::Hands;
pub use open_d::{OpenD, PreTrainD};
pub use spec::Spec;
pub use state::State;

/// Shared memory name constants matching the C# BIDSSMemLib.
pub mod names {
    /// BIDSSharedMemoryData
    pub const BSMD: &str = "BIDSSharedMemory";
    /// OpenD (OpenBVE extended data)
    pub const OPEN_D: &str = "BIDSSharedMemoryO";
    /// Panel data (int array)
    pub const PANEL: &str = "BIDSSharedMemoryPn";
    /// Sound data (int array)
    pub const SOUND: &str = "BIDSSharedMemorySn";
    /// Key input (128 bools)
    pub const CTRL_KEY: &str = "BIDSSMemCtrlK";
    /// Handle input (Hands struct)
    pub const CTRL_HAND: &str = "BIDSSMemCtrlH";
}

/// Page size for shared memory allocation, matching the C# implementation.
pub const PAGE_SIZE: u64 = 4096;

/// Current BIDS data structure version.
pub const VERSION: i32 = 203;
