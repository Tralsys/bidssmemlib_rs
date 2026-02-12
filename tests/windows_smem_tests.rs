#![cfg(windows)]

use bids_smemlib::smem::{SMemIF, SMemIFExt};
use bids_smemlib::types::*;

#[test]
fn test_windows_smem_create_and_read_write() {
    let smem =
        bids_smemlib::smem::windows::WindowsSMemIF::open_or_create("RustTest_Basic", 4096).unwrap();

    let spec = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    smem.write_value(0, &spec).unwrap();
    let read_back: Spec = smem.read_value(0).unwrap();
    assert_eq!(spec, read_back);
}

#[test]
fn test_windows_smem_bsmd() {
    let smem =
        bids_smemlib::smem::windows::WindowsSMemIF::open_or_create("RustTest_BSMD", 4096).unwrap();

    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    bsmd.version_num = VERSION;
    bsmd.spec_data = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    bsmd.state_data = State {
        z: 12345.678,
        v: 80.5,
        t: 43200000,
        bc: 200.0,
        mr: 780.0,
        er: 490.0,
        bp: 490.0,
        sap: 490.0,
        i: 150.0,
    };
    bsmd.handle_data = Hand {
        b: 0,
        p: 3,
        r: 1,
        c: 0,
    };
    bsmd.set_is_door_closed(true);

    smem.write_value(0, &bsmd).unwrap();
    let read_back: BIDSSharedMemoryData = smem.read_value(0).unwrap();
    assert_eq!(bsmd, read_back);
    assert!(read_back.is_enabled());
    assert!(read_back.is_door_closed());
}

#[test]
fn test_windows_smem_shared_between_handles() {
    let name = "RustTest_Shared";

    let smem1 = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(name, 4096).unwrap();
    let smem2 = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(name, 4096).unwrap();

    // smem1 is the creator, smem2 opens existing
    assert!(smem1.is_newly_created());
    assert!(!smem2.is_newly_created());

    // Write via smem1, read via smem2
    let spec = Spec {
        b: 10,
        p: 6,
        a: 3,
        j: 8,
        c: 12,
    };
    smem1.write_value(0, &spec).unwrap();
    let read_back: Spec = smem2.read_value(0).unwrap();
    assert_eq!(spec, read_back);

    // Write via smem2, read via smem1
    let hand = Hand {
        b: 5,
        p: 3,
        r: -1,
        c: 2,
    };
    smem2.write_value(100, &hand).unwrap();
    let read_back: Hand = smem1.read_value(100).unwrap();
    assert_eq!(hand, read_back);
}

#[test]
fn test_windows_smem_array() {
    let smem =
        bids_smemlib::smem::windows::WindowsSMemIF::open_or_create("RustTest_Array", 4096).unwrap();

    let data: Vec<i32> = (0..256).collect();
    smem.write_array(0, &data).unwrap();
    let read_back: Vec<i32> = smem.read_array(0, 256).unwrap();
    assert_eq!(data, read_back);
}

#[test]
fn test_windows_smem_lib_full_round_trip() {
    // Use the high-level SMemLib with real Windows shared memory
    let lib = bids_smemlib::smem_lib::SMemLib::new().unwrap();

    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    bsmd.version_num = VERSION;
    bsmd.spec_data = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    bsmd.state_data = State {
        z: 5000.0,
        v: 120.0,
        t: 43200000,
        ..State::default()
    };
    bsmd.handle_data = Hand {
        b: 0,
        p: 4,
        r: 1,
        c: 0,
    };
    bsmd.set_is_door_closed(true);

    lib.write_bsmd(&bsmd).unwrap();
    let read_back = lib.read_bsmd().unwrap();
    assert_eq!(bsmd, read_back);

    // Panel
    let panel: Vec<i32> = (0..256).collect();
    lib.write_panel(&panel).unwrap();
    assert_eq!(lib.read_panel().unwrap(), panel);

    // Sound
    let sound: Vec<i32> = vec![0, 1, -1000, 2];
    lib.write_sound(&sound).unwrap();
    assert_eq!(lib.read_sound().unwrap(), sound);
}
