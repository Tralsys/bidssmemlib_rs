use bids_smemlib::types::*;
use static_assertions::{assert_eq_align, assert_eq_size};

// ---- Size assertions ----
assert_eq_size!(Spec, [u8; 20]);
assert_eq_size!(State, [u8; 40]);
assert_eq_size!(Hand, [u8; 16]);
assert_eq_size!(Hands, [u8; 32]);
assert_eq_size!(PreTrainD, [u8; 32]);
assert_eq_size!(OpenD, [u8; 80]);
assert_eq_size!(BIDSSharedMemoryData, [u8; 96]);

// ---- Alignment assertions ----
assert_eq_align!(Spec, i32);
assert_eq_align!(State, f64);
assert_eq_align!(Hand, i32);
assert_eq_align!(Hands, f64);
assert_eq_align!(PreTrainD, f64);
assert_eq_align!(OpenD, f64);
assert_eq_align!(BIDSSharedMemoryData, f64);

// ---- Field offset tests ----

#[test]
fn test_spec_offsets() {
    assert_eq!(std::mem::offset_of!(Spec, b), 0);
    assert_eq!(std::mem::offset_of!(Spec, p), 4);
    assert_eq!(std::mem::offset_of!(Spec, a), 8);
    assert_eq!(std::mem::offset_of!(Spec, j), 12);
    assert_eq!(std::mem::offset_of!(Spec, c), 16);
}

#[test]
fn test_state_offsets() {
    assert_eq!(std::mem::offset_of!(State, z), 0);
    assert_eq!(std::mem::offset_of!(State, v), 8);
    assert_eq!(std::mem::offset_of!(State, t), 12);
    assert_eq!(std::mem::offset_of!(State, bc), 16);
    assert_eq!(std::mem::offset_of!(State, mr), 20);
    assert_eq!(std::mem::offset_of!(State, er), 24);
    assert_eq!(std::mem::offset_of!(State, bp), 28);
    assert_eq!(std::mem::offset_of!(State, sap), 32);
    assert_eq!(std::mem::offset_of!(State, i), 36);
}

#[test]
fn test_hand_offsets() {
    assert_eq!(std::mem::offset_of!(Hand, b), 0);
    assert_eq!(std::mem::offset_of!(Hand, p), 4);
    assert_eq!(std::mem::offset_of!(Hand, r), 8);
    assert_eq!(std::mem::offset_of!(Hand, c), 12);
}

#[test]
fn test_hands_offsets() {
    assert_eq!(std::mem::offset_of!(Hands, b), 0);
    assert_eq!(std::mem::offset_of!(Hands, p), 4);
    assert_eq!(std::mem::offset_of!(Hands, r), 8);
    assert_eq!(std::mem::offset_of!(Hands, s), 12);
    assert_eq!(std::mem::offset_of!(Hands, b_pos), 16);
    assert_eq!(std::mem::offset_of!(Hands, p_pos), 24);
}

#[test]
fn test_pre_train_d_offsets() {
    assert_eq!(std::mem::offset_of!(PreTrainD, location), 8);
    assert_eq!(std::mem::offset_of!(PreTrainD, distance), 16);
    assert_eq!(std::mem::offset_of!(PreTrainD, speed), 24);
}

#[test]
fn test_open_d_offsets() {
    assert_eq!(std::mem::offset_of!(OpenD, ver), 4);
    assert_eq!(std::mem::offset_of!(OpenD, radius), 8);
    assert_eq!(std::mem::offset_of!(OpenD, cant), 16);
    assert_eq!(std::mem::offset_of!(OpenD, pitch), 24);
    assert_eq!(std::mem::offset_of!(OpenD, elap_time), 32);
    assert_eq!(std::mem::offset_of!(OpenD, pre_train), 40);
    assert_eq!(std::mem::offset_of!(OpenD, self_b_count), 72);
    assert_eq!(std::mem::offset_of!(OpenD, self_b_position), 76);
}

#[test]
fn test_bsmd_offsets() {
    assert_eq!(std::mem::offset_of!(BIDSSharedMemoryData, version_num), 4);
    assert_eq!(std::mem::offset_of!(BIDSSharedMemoryData, spec_data), 8);
    assert_eq!(std::mem::offset_of!(BIDSSharedMemoryData, state_data), 32);
    assert_eq!(std::mem::offset_of!(BIDSSharedMemoryData, handle_data), 72);
}

// ---- Round-trip byte casting ----

#[test]
fn test_spec_round_trip() {
    let spec = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    let bytes = bytemuck::bytes_of(&spec);
    let restored: &Spec = bytemuck::from_bytes(bytes);
    assert_eq!(&spec, restored);
}

#[test]
fn test_state_round_trip() {
    let state = State {
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
    let bytes = bytemuck::bytes_of(&state);
    let restored: &State = bytemuck::from_bytes(bytes);
    assert_eq!(&state, restored);
}

#[test]
fn test_bsmd_round_trip() {
    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    bsmd.version_num = 203;
    bsmd.spec_data = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    bsmd.state_data = State {
        z: 1000.0,
        v: 60.0,
        t: 36000000,
        bc: 0.0,
        mr: 780.0,
        er: 490.0,
        bp: 490.0,
        sap: 490.0,
        i: 100.0,
    };
    bsmd.handle_data = Hand {
        b: 0,
        p: 3,
        r: 1,
        c: 0,
    };
    bsmd.set_is_door_closed(true);

    let bytes = bytemuck::bytes_of(&bsmd);
    let restored: &BIDSSharedMemoryData = bytemuck::from_bytes(bytes);
    assert_eq!(&bsmd, restored);
    assert!(restored.is_enabled());
    assert!(restored.is_door_closed());
    assert_eq!(restored.version_num, 203);
}

#[test]
fn test_open_d_round_trip() {
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

    let bytes = bytemuck::bytes_of(&open_d);
    let restored: &OpenD = bytemuck::from_bytes(bytes);
    assert_eq!(&open_d, restored);
    assert!(restored.is_enabled());
    assert!(restored.pre_train.is_enabled());
}

#[test]
fn test_hands_round_trip() {
    let hands = Hands {
        b: 3,
        p: 0,
        r: 1,
        s: 2,
        b_pos: 0.375,
        p_pos: 0.0,
    };
    let bytes = bytemuck::bytes_of(&hands);
    let restored: &Hands = bytemuck::from_bytes(bytes);
    assert_eq!(&hands, restored);
}

// ---- Bool byte representation ----

#[test]
fn test_bool_is_single_byte() {
    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    let bytes = bytemuck::bytes_of(&bsmd);
    // is_enabled is at offset 0, should be 0x01
    assert_eq!(bytes[0], 1);
    // padding bytes should be zero
    assert_eq!(bytes[1], 0);
    assert_eq!(bytes[2], 0);
    assert_eq!(bytes[3], 0);

    bsmd.set_is_enabled(false);
    let bytes = bytemuck::bytes_of(&bsmd);
    assert_eq!(bytes[0], 0);
}

// ---- Constants ----

#[test]
fn test_constants() {
    assert_eq!(bids_smemlib::types::VERSION, 203);
    assert_eq!(bids_smemlib::types::PAGE_SIZE, 4096);
    assert_eq!(bids_smemlib::types::names::BSMD, "BIDSSharedMemory");
    assert_eq!(bids_smemlib::types::names::OPEN_D, "BIDSSharedMemoryO");
    assert_eq!(bids_smemlib::types::names::PANEL, "BIDSSharedMemoryPn");
    assert_eq!(bids_smemlib::types::names::SOUND, "BIDSSharedMemorySn");
    assert_eq!(bids_smemlib::types::names::CTRL_KEY, "BIDSSMemCtrlK");
    assert_eq!(bids_smemlib::types::names::CTRL_HAND, "BIDSSMemCtrlH");
}
