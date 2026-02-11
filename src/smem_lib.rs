use crate::controller::{ArrayDataSMemCtrler, SMemCtrler};
use crate::error::SMemResult;
use crate::types::*;

/// High-level BIDS shared memory library.
///
/// Provides convenient access to all BIDS shared memory regions:
/// - `BSMD` (BIDSSharedMemoryData): main vehicle data
/// - `OpenD`: OpenBVE extended data
/// - `Panel`: panel data (int array, up to 256 elements)
/// - `Sound`: sound data (int array, up to 256 elements)
///
/// Equivalent to C# `SMemLib` / `StaticSMemLib`.
pub struct SMemLib {
    smc_bsmd: SMemCtrler<BIDSSharedMemoryData>,
    smc_open_d: SMemCtrler<OpenD>,
    smc_panel: ArrayDataSMemCtrler<i32>,
    smc_sound: ArrayDataSMemCtrler<i32>,
}

impl SMemLib {
    /// Create a new SMemLib using platform shared memory (Windows: MMF, others: in-memory).
    pub fn new() -> SMemResult<Self> {
        Ok(Self {
            smc_bsmd: SMemCtrler::new(names::BSMD)?,
            smc_open_d: SMemCtrler::new(names::OPEN_D)?,
            smc_panel: ArrayDataSMemCtrler::new(names::PANEL)?,
            smc_sound: ArrayDataSMemCtrler::new(names::SOUND)?,
        })
    }

    /// Create a new SMemLib using in-memory storage (no IPC, useful for testing).
    pub fn in_memory() -> Self {
        Self {
            smc_bsmd: SMemCtrler::in_memory(names::BSMD),
            smc_open_d: SMemCtrler::in_memory(names::OPEN_D),
            smc_panel: ArrayDataSMemCtrler::in_memory(names::PANEL),
            smc_sound: ArrayDataSMemCtrler::in_memory(names::SOUND),
        }
    }

    // ---- BSMD ----

    /// Read BIDSSharedMemoryData from shared memory.
    pub fn read_bsmd(&self) -> SMemResult<BIDSSharedMemoryData> {
        self.smc_bsmd.read()
    }

    /// Write BIDSSharedMemoryData to shared memory.
    pub fn write_bsmd(&self, data: &BIDSSharedMemoryData) -> SMemResult<()> {
        self.smc_bsmd.write(data)
    }

    /// Write only the Spec field of BSMD (partial update).
    pub fn write_spec(&self, spec: &Spec) -> SMemResult<()> {
        let offset = std::mem::offset_of!(BIDSSharedMemoryData, spec_data) as u64;
        self.smc_bsmd.write_field(offset, spec)
    }

    /// Write only the State field of BSMD (partial update).
    pub fn write_state(&self, state: &State) -> SMemResult<()> {
        let offset = std::mem::offset_of!(BIDSSharedMemoryData, state_data) as u64;
        self.smc_bsmd.write_field(offset, state)
    }

    /// Write only the Hand field of BSMD (partial update).
    pub fn write_handle(&self, hand: &Hand) -> SMemResult<()> {
        let offset = std::mem::offset_of!(BIDSSharedMemoryData, handle_data) as u64;
        self.smc_bsmd.write_field(offset, hand)
    }

    /// Write only the version_num field of BSMD (partial update).
    pub fn write_version(&self, ver: i32) -> SMemResult<()> {
        let offset = std::mem::offset_of!(BIDSSharedMemoryData, version_num) as u64;
        self.smc_bsmd.write_field(offset, &ver)
    }

    // ---- OpenD ----

    /// Read OpenD from shared memory.
    pub fn read_open_d(&self) -> SMemResult<OpenD> {
        self.smc_open_d.read()
    }

    /// Write OpenD to shared memory.
    pub fn write_open_d(&self, data: &OpenD) -> SMemResult<()> {
        self.smc_open_d.write(data)
    }

    // ---- Panel ----

    /// Read panel data array from shared memory.
    pub fn read_panel(&self) -> SMemResult<Vec<i32>> {
        self.smc_panel.read()
    }

    /// Write panel data array to shared memory.
    pub fn write_panel(&self, data: &[i32]) -> SMemResult<()> {
        self.smc_panel.write(data)
    }

    // ---- Sound ----

    /// Read sound data array from shared memory.
    pub fn read_sound(&self) -> SMemResult<Vec<i32>> {
        self.smc_sound.read()
    }

    /// Write sound data array to shared memory.
    pub fn write_sound(&self, data: &[i32]) -> SMemResult<()> {
        self.smc_sound.write(data)
    }
}
