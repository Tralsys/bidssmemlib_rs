use bids_smemlib::smem::in_memory::InMemorySMemIF;
use bids_smemlib::smem::SMemIFExt;
use bids_smemlib::types::*;

#[test]
fn test_read_write_struct() {
    let smem = InMemorySMemIF::new("test_struct", 4096);

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
fn test_read_write_bsmd() {
    let smem = InMemorySMemIF::new("test_bsmd", 4096);

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
        bc: 0.0,
        mr: 780.0,
        er: 490.0,
        bp: 490.0,
        sap: 490.0,
        i: 200.0,
    };
    bsmd.handle_data = Hand {
        b: 0,
        p: 4,
        r: 1,
        c: 0,
    };
    bsmd.set_is_door_closed(true);

    smem.write_value(0, &bsmd).unwrap();
    let read_back: BIDSSharedMemoryData = smem.read_value(0).unwrap();
    assert_eq!(bsmd, read_back);
    assert!(read_back.is_enabled());
    assert!(read_back.is_door_closed());
    assert_eq!(read_back.version_num, VERSION);
}

#[test]
fn test_read_write_open_d() {
    let smem = InMemorySMemIF::new("test_open_d", 4096);

    let mut open_d = OpenD::default();
    open_d.set_is_enabled(true);
    open_d.radius = 300.0;
    open_d.cant = 80.0;
    open_d.pitch = 10.0;
    open_d.elap_time = 33.33;
    open_d.pre_train.set_is_enabled(true);
    open_d.pre_train.location = 10000.0;
    open_d.pre_train.distance = 500.0;
    open_d.pre_train.speed = 85.0;
    open_d.self_b_count = 3;
    open_d.self_b_position = 1;

    smem.write_value(0, &open_d).unwrap();
    let read_back: OpenD = smem.read_value(0).unwrap();
    assert_eq!(open_d, read_back);
}

#[test]
fn test_read_write_array() {
    let smem = InMemorySMemIF::new("test_array", 4096);

    let data: Vec<i32> = (0..256).collect();
    smem.write_array(0, &data).unwrap();
    let read_back: Vec<i32> = smem.read_array(0, 256).unwrap();
    assert_eq!(data, read_back);
}

#[test]
fn test_read_write_at_offset() {
    let smem = InMemorySMemIF::new("test_offset", 4096);

    let spec = Spec {
        b: 1,
        p: 2,
        a: 3,
        j: 4,
        c: 5,
    };
    smem.write_value(100, &spec).unwrap();
    let read_back: Spec = smem.read_value(100).unwrap();
    assert_eq!(spec, read_back);
}

#[test]
fn test_out_of_bounds_read() {
    let smem = InMemorySMemIF::new("test_oob", 16);
    let result: Result<Spec, _> = smem.read_value(0);
    // Spec is 20 bytes, capacity is 16 -> should fail
    assert!(result.is_err());
}

#[test]
fn test_out_of_bounds_write() {
    let smem = InMemorySMemIF::new("test_oob_write", 16);
    let spec = Spec::default();
    let result = smem.write_value(0, &spec);
    assert!(result.is_err());
}

#[test]
fn test_concurrent_read_write() {
    use std::sync::Arc;
    use std::thread;

    let smem = Arc::new(InMemorySMemIF::new("test_concurrent", 4096));

    let writer = {
        let smem = Arc::clone(&smem);
        thread::spawn(move || {
            for i in 0..100 {
                let val: i32 = i;
                smem.write_value(0, &val).unwrap();
            }
        })
    };

    let reader = {
        let smem = Arc::clone(&smem);
        thread::spawn(move || {
            for _ in 0..100 {
                let _val: i32 = smem.read_value(0).unwrap();
            }
        })
    };

    writer.join().unwrap();
    reader.join().unwrap();
}

#[test]
fn test_zero_length_array() {
    let smem = InMemorySMemIF::new("test_zero", 4096);
    let empty: Vec<i32> = Vec::new();
    smem.write_array(0, &empty).unwrap();
    let read_back: Vec<i32> = smem.read_array(0, 0).unwrap();
    assert!(read_back.is_empty());
}

#[test]
fn test_hands_read_write() {
    let smem = InMemorySMemIF::new("test_hands", 4096);

    let hands = Hands {
        b: 5,
        p: 3,
        r: -1,
        s: 0,
        b_pos: 0.625,
        p_pos: 0.6,
    };
    smem.write_value(0, &hands).unwrap();
    let read_back: Hands = smem.read_value(0).unwrap();
    assert_eq!(hands, read_back);
}
