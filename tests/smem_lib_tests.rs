use bids_smemlib::smem_lib::SMemLib;
use bids_smemlib::types::*;

#[test]
fn test_smem_lib_bsmd_round_trip() {
    let lib = SMemLib::in_memory();

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
        z: 12345.0,
        v: 90.0,
        t: 43200000,
        bc: 0.0,
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

    lib.write_bsmd(&bsmd).unwrap();
    let read_back = lib.read_bsmd().unwrap();
    assert_eq!(bsmd, read_back);
}

#[test]
fn test_smem_lib_partial_writes() {
    let lib = SMemLib::in_memory();

    // Write initial BSMD
    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    bsmd.version_num = VERSION;
    lib.write_bsmd(&bsmd).unwrap();

    // Partial write spec
    let spec = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    lib.write_spec(&spec).unwrap();

    // Partial write state
    let state = State {
        z: 1000.0,
        v: 60.0,
        t: 36000000,
        bc: 100.0,
        mr: 780.0,
        er: 490.0,
        bp: 490.0,
        sap: 490.0,
        i: 50.0,
    };
    lib.write_state(&state).unwrap();

    // Partial write handle
    let hand = Hand {
        b: 2,
        p: 0,
        r: -1,
        c: 0,
    };
    lib.write_handle(&hand).unwrap();

    // Partial write version
    lib.write_version(999).unwrap();

    // Read back and verify all fields
    let read_back = lib.read_bsmd().unwrap();
    assert!(read_back.is_enabled());
    assert_eq!(read_back.version_num, 999);
    assert_eq!(read_back.spec_data, spec);
    assert_eq!(read_back.state_data, state);
    assert_eq!(read_back.handle_data, hand);
}

#[test]
fn test_smem_lib_open_d() {
    let lib = SMemLib::in_memory();

    let mut open_d = OpenD::default();
    open_d.set_is_enabled(true);
    open_d.radius = 500.0;
    open_d.cant = 105.0;
    open_d.pitch = -15.0;
    open_d.elap_time = 16.67;
    open_d.pre_train.set_is_enabled(true);
    open_d.pre_train.location = 5000.0;
    open_d.pre_train.distance = 1200.0;
    open_d.pre_train.speed = 75.0;
    open_d.self_b_count = 5;
    open_d.self_b_position = 2;

    lib.write_open_d(&open_d).unwrap();
    let read_back = lib.read_open_d().unwrap();
    assert_eq!(open_d, read_back);
}

#[test]
fn test_smem_lib_panel() {
    let lib = SMemLib::in_memory();

    let panel: Vec<i32> = (0..256).collect();
    lib.write_panel(&panel).unwrap();
    let read_back = lib.read_panel().unwrap();
    assert_eq!(panel, read_back);
}

#[test]
fn test_smem_lib_sound() {
    let lib = SMemLib::in_memory();

    let sound: Vec<i32> = vec![0, 1, 2, -1000, 0, 1, 0, -1000];
    lib.write_sound(&sound).unwrap();
    let read_back = lib.read_sound().unwrap();
    assert_eq!(sound, read_back);
}

#[test]
fn test_smem_lib_empty_panel() {
    let lib = SMemLib::in_memory();

    lib.write_panel(&[]).unwrap();
    let read_back = lib.read_panel().unwrap();
    assert!(read_back.is_empty());
}

// ---- CtrlInput tests ----

use bids_smemlib::ctrl_input::CtrlInput;

#[test]
fn test_ctrl_input_keys() {
    let ctrl = CtrlInput::in_memory();

    let mut keys = [false; 128];
    keys[0] = true;
    keys[5] = true;
    keys[15] = true;
    keys[127] = true;

    ctrl.write_keys(&keys).unwrap();
    let read_back = ctrl.read_keys().unwrap();
    assert_eq!(keys, read_back);
}

#[test]
fn test_ctrl_input_single_key() {
    let ctrl = CtrlInput::in_memory();

    // Initialize with all false
    let keys = [false; 128];
    ctrl.write_keys(&keys).unwrap();

    // Write single key
    ctrl.write_key(10, true).unwrap();
    assert!(ctrl.read_key(10).unwrap());
    assert!(!ctrl.read_key(0).unwrap());
    assert!(!ctrl.read_key(127).unwrap());
}

#[test]
fn test_ctrl_input_hands() {
    let ctrl = CtrlInput::in_memory();

    let hands = Hands {
        b: 5,
        p: 3,
        r: 1,
        s: 0,
        b_pos: 0.625,
        p_pos: 0.6,
    };

    ctrl.write_hands(&hands).unwrap();
    let read_back = ctrl.read_hands().unwrap();
    assert_eq!(hands, read_back);
}

#[test]
fn test_ctrl_input_key_out_of_range() {
    let ctrl = CtrlInput::in_memory();
    assert!(ctrl.read_key(128).is_err());
    assert!(ctrl.write_key(128, true).is_err());
}
