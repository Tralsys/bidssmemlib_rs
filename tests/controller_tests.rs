use bids_smemlib::controller::{ArrayDataSMemCtrler, SMemCtrler};
use bids_smemlib::types::*;

// ---- SMemCtrler tests ----

#[test]
fn test_struct_ctrler_write_read() {
    let ctrler: SMemCtrler<Spec> = SMemCtrler::in_memory("test_ctrler_spec");

    let spec = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    ctrler.write(&spec).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(spec, read_back);
}

#[test]
fn test_struct_ctrler_cached_value() {
    let ctrler: SMemCtrler<Spec> = SMemCtrler::in_memory("test_cached");

    let spec = Spec {
        b: 1,
        p: 2,
        a: 3,
        j: 4,
        c: 5,
    };
    ctrler.write(&spec).unwrap();
    assert_eq!(ctrler.cached_value(), spec);
}

#[test]
fn test_struct_ctrler_bsmd() {
    let ctrler: SMemCtrler<BIDSSharedMemoryData> = SMemCtrler::in_memory("test_ctrler_bsmd");

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
    bsmd.state_data.z = 5000.0;
    bsmd.state_data.v = 120.0;
    bsmd.handle_data.p = 4;
    bsmd.handle_data.r = 1;
    bsmd.set_is_door_closed(true);

    ctrler.write(&bsmd).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(bsmd, read_back);
    assert!(read_back.is_enabled());
    assert!(read_back.is_door_closed());
}

#[test]
fn test_struct_ctrler_partial_write() {
    let ctrler: SMemCtrler<BIDSSharedMemoryData> = SMemCtrler::in_memory("test_partial");

    // Write initial BSMD
    let mut bsmd = BIDSSharedMemoryData::default();
    bsmd.set_is_enabled(true);
    bsmd.version_num = VERSION;
    ctrler.write(&bsmd).unwrap();

    // Partial write: update just the Spec
    let spec = Spec {
        b: 8,
        p: 5,
        a: 2,
        j: 7,
        c: 10,
    };
    let offset = std::mem::offset_of!(BIDSSharedMemoryData, spec_data) as u64;
    ctrler.write_field(offset, &spec).unwrap();

    // Read back and verify
    let read_back = ctrler.read().unwrap();
    assert_eq!(read_back.spec_data, spec);
    assert!(read_back.is_enabled());
    assert_eq!(read_back.version_num, VERSION);
}

#[test]
fn test_struct_ctrler_try_read_write() {
    let ctrler: SMemCtrler<Hand> = SMemCtrler::in_memory("test_try");

    let hand = Hand {
        b: 3,
        p: 2,
        r: 1,
        c: 0,
    };
    assert!(ctrler.try_write(&hand));
    let read_back = ctrler.try_read().unwrap();
    assert_eq!(hand, read_back);
}

#[test]
fn test_struct_ctrler_no_smem() {
    let ctrler: SMemCtrler<Spec> = SMemCtrler::no_smem();
    assert!(ctrler.read().is_err());
    assert!(ctrler.try_read().is_none());
}

// ---- ArrayDataSMemCtrler tests ----

#[test]
fn test_array_ctrler_write_read() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr");

    let data: Vec<i32> = (0..100).collect();
    ctrler.write(&data).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(data, read_back);
}

#[test]
fn test_array_ctrler_empty() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_empty");

    let data: Vec<i32> = Vec::new();
    ctrler.write(&data).unwrap();
    let read_back = ctrler.read().unwrap();
    assert!(read_back.is_empty());
}

#[test]
fn test_array_ctrler_256_elements() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_256");

    let data: Vec<i32> = (0..256).collect();
    ctrler.write(&data).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(data, read_back);
}

#[test]
fn test_array_ctrler_read_at() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_at");

    let data: Vec<i32> = (100..200).collect();
    ctrler.write(&data).unwrap();

    assert_eq!(ctrler.read_at(0).unwrap(), 100);
    assert_eq!(ctrler.read_at(50).unwrap(), 150);
    assert_eq!(ctrler.read_at(99).unwrap(), 199);
}

#[test]
fn test_array_ctrler_write_at() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_write_at");

    let data: Vec<i32> = vec![0; 10];
    ctrler.write(&data).unwrap();

    ctrler.write_at(5, &999).unwrap();
    assert_eq!(ctrler.read_at(5).unwrap(), 999);
    // Other elements should be unchanged
    assert_eq!(ctrler.read_at(0).unwrap(), 0);
    assert_eq!(ctrler.read_at(9).unwrap(), 0);
}

#[test]
fn test_array_ctrler_index_out_of_range() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_oob");

    let data: Vec<i32> = vec![1, 2, 3];
    ctrler.write(&data).unwrap();

    assert!(ctrler.read_at(3).is_err());
    assert!(ctrler.write_at(3, &10).is_err());
}

#[test]
fn test_array_ctrler_cached_value() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_cached");

    let data: Vec<i32> = vec![10, 20, 30];
    ctrler.write(&data).unwrap();
    assert_eq!(ctrler.cached_value(), data);
}

#[test]
fn test_array_ctrler_overwrite() {
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_overwrite");

    let data1: Vec<i32> = vec![1, 2, 3, 4, 5];
    ctrler.write(&data1).unwrap();
    assert_eq!(ctrler.read().unwrap(), data1);

    let data2: Vec<i32> = vec![10, 20];
    ctrler.write(&data2).unwrap();
    assert_eq!(ctrler.read().unwrap(), data2);
}

#[test]
fn test_array_ctrler_u8_keys() {
    let ctrler: ArrayDataSMemCtrler<u8> = ArrayDataSMemCtrler::in_memory("test_keys");

    let mut keys = vec![0u8; 128];
    keys[0] = 1;
    keys[5] = 1;
    keys[127] = 1;

    ctrler.write(&keys).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(keys, read_back);
}

#[test]
fn test_array_ctrler_max_inline() {
    // For i32 elements: max inline = (4096 - 4) / 4 = 1023
    let ctrler: ArrayDataSMemCtrler<i32> = ArrayDataSMemCtrler::in_memory("test_arr_max_inline");

    let data: Vec<i32> = (0..1023).collect();
    ctrler.write(&data).unwrap();
    let read_back = ctrler.read().unwrap();
    assert_eq!(data, read_back);
}
