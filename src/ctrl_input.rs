use crate::controller::{ArrayDataSMemCtrler, SMemCtrler};
use crate::error::{SMemError, SMemResult};
use crate::types::*;

/// Maximum number of key inputs.
pub const KEY_ARRAY_SIZE_MAX: usize = 128;

/// Input control for BIDS shared memory.
///
/// Provides access to:
/// - Key input: 128 boolean values stored as `u8` (0 or 1) in shared memory.
/// - Handle input: `Hands` struct for extended handle data.
///
/// Equivalent to C# `CtrlInput`.
pub struct CtrlInput {
    smc_key: ArrayDataSMemCtrler<u8>,
    smc_hand: SMemCtrler<Hands>,
}

impl CtrlInput {
    /// Create a new CtrlInput using platform shared memory.
    pub fn new() -> SMemResult<Self> {
        Ok(Self {
            smc_key: ArrayDataSMemCtrler::new(names::CTRL_KEY)?,
            smc_hand: SMemCtrler::new(names::CTRL_HAND)?,
        })
    }

    /// Create a new CtrlInput using in-memory storage (no IPC).
    pub fn in_memory() -> Self {
        Self {
            smc_key: ArrayDataSMemCtrler::in_memory(names::CTRL_KEY),
            smc_hand: SMemCtrler::in_memory(names::CTRL_HAND),
        }
    }

    // ---- Keys ----

    /// Read all key states from shared memory.
    pub fn read_keys(&self) -> SMemResult<[bool; KEY_ARRAY_SIZE_MAX]> {
        let raw = self.smc_key.read()?;
        let mut result = [false; KEY_ARRAY_SIZE_MAX];
        for (i, &v) in raw.iter().enumerate().take(KEY_ARRAY_SIZE_MAX) {
            result[i] = v != 0;
        }
        Ok(result)
    }

    /// Write all key states to shared memory.
    pub fn write_keys(&self, keys: &[bool]) -> SMemResult<()> {
        let raw: Vec<u8> = keys
            .iter()
            .take(KEY_ARRAY_SIZE_MAX)
            .map(|&b| b as u8)
            .collect();
        self.smc_key.write(&raw)
    }

    /// Read a single key state by index.
    pub fn read_key(&self, index: usize) -> SMemResult<bool> {
        if index >= KEY_ARRAY_SIZE_MAX {
            return Err(SMemError::IndexOutOfRange {
                index,
                length: KEY_ARRAY_SIZE_MAX,
            });
        }
        let v: u8 = self.smc_key.read_at(index)?;
        Ok(v != 0)
    }

    /// Write a single key state by index.
    pub fn write_key(&self, index: usize, value: bool) -> SMemResult<()> {
        if index >= KEY_ARRAY_SIZE_MAX {
            return Err(SMemError::IndexOutOfRange {
                index,
                length: KEY_ARRAY_SIZE_MAX,
            });
        }
        self.smc_key.write_at(index, &(value as u8))
    }

    // ---- Hands ----

    /// Read the handle data from shared memory.
    pub fn read_hands(&self) -> SMemResult<Hands> {
        self.smc_hand.read()
    }

    /// Write the handle data to shared memory.
    pub fn write_hands(&self, hands: &Hands) -> SMemResult<()> {
        self.smc_hand.write(hands)
    }
}
