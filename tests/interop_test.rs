//! C# <-> Rust interop tests.
//!
//! These tests verify binary layout compatibility by having C# and Rust
//! read/write the same named shared memory regions.
//!
//! Only runs on Windows (shared memory is required for IPC).
//! Requires .NET 8 SDK to be installed (`dotnet` on PATH).
#![cfg(windows)]

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use bids_smemlib::smem::SMemIFExt;
use bids_smemlib::types::*;

/// Path to the C# interop test project.
fn csharp_project_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/csharp_interop")
}

/// Build the C# interop test project. Returns true if successful.
fn build_csharp() -> bool {
    let status = Command::new("dotnet")
        .args(["build", "-c", "Release", "--nologo", "-v", "quiet"])
        .current_dir(csharp_project_dir())
        .status();
    match status {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("Failed to run dotnet build: {e}");
            false
        }
    }
}

/// Run the C# interop test with the given arguments.
/// Returns a child process with stdin/stdout captured.
fn run_csharp(args: &[&str]) -> std::process::Child {
    Command::new("dotnet")
        .args(["run", "-c", "Release", "--no-build", "--project"])
        .arg(csharp_project_dir().to_str().unwrap())
        .args(["--"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn C# interop test process")
}

/// Run the C# interop test and wait for completion.
fn run_csharp_and_wait(args: &[&str]) -> (i32, String, String) {
    let output = Command::new("dotnet")
        .args(["run", "-c", "Release", "--no-build", "--project"])
        .arg(csharp_project_dir().to_str().unwrap())
        .args(["--"])
        .args(args)
        .output()
        .expect("Failed to run C# interop test");

    let code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (code, stdout, stderr)
}

/// Wait for the C# process to print "READY" on stdout.
fn wait_for_ready(reader: &mut BufReader<std::process::ChildStdout>) {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .expect("Failed to read from C# stdout");
    assert!(
        line.trim() == "READY",
        "Expected 'READY' from C#, got: {line:?}"
    );
}

/// Send "QUIT" to the C# process's stdin to tell it to exit.
fn send_quit(stdin: &mut std::process::ChildStdin) {
    writeln!(stdin, "QUIT").expect("Failed to write QUIT to C# stdin");
    stdin.flush().expect("Failed to flush C# stdin");
}

// Well-known test data (must match Program.cs exactly)

fn test_bsmd() -> BIDSSharedMemoryData {
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
    bsmd
}

fn test_open_d() -> OpenD {
    let mut open_d = OpenD::default();
    open_d.set_is_enabled(true);
    open_d.ver = 1;
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
    open_d
}

fn test_panel() -> Vec<i32> {
    (0..256).map(|i| i * 10).collect()
}

// ---- Test: C# writes BSMD, Rust reads ----

#[test]
fn test_csharp_writes_bsmd_rust_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_CSharpWritesBsmd";
    let mut child = run_csharp(&["write-bsmd", mmf_name]);

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    wait_for_ready(&mut reader);

    // Open the same shared memory from Rust and read
    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();
    let bsmd: BIDSSharedMemoryData = smem.read_value(0).unwrap();

    let expected = test_bsmd();
    assert_eq!(bsmd.is_enabled(), expected.is_enabled());
    assert_eq!(bsmd.version_num, expected.version_num);
    assert_eq!(bsmd.spec_data, expected.spec_data);
    assert_eq!(bsmd.state_data, expected.state_data);
    assert_eq!(bsmd.handle_data, expected.handle_data);
    assert_eq!(bsmd.is_door_closed(), expected.is_door_closed());
    assert_eq!(bsmd, expected);

    send_quit(&mut child.stdin.take().unwrap());
    let status = child.wait().unwrap();
    assert!(status.success(), "C# process exited with error");
}

// ---- Test: Rust writes BSMD, C# reads ----

#[test]
fn test_rust_writes_bsmd_csharp_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_RustWritesBsmd";

    // Create shared memory from Rust and write test data
    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();
    let bsmd = test_bsmd();
    smem.write_value(0, &bsmd).unwrap();

    // Launch C# to verify the data
    let (code, stdout, stderr) = run_csharp_and_wait(&["verify-bsmd", mmf_name]);

    eprintln!("C# stdout: {stdout}");
    if !stderr.is_empty() {
        eprintln!("C# stderr: {stderr}");
    }
    assert_eq!(code, 0, "C# verification failed");
    assert!(stdout.contains("OK"), "C# output should contain OK");

    drop(smem);
}

// ---- Test: C# writes OpenD, Rust reads ----

#[test]
fn test_csharp_writes_open_d_rust_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_CSharpWritesOpenD";
    let mut child = run_csharp(&["write-open-d", mmf_name]);

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    wait_for_ready(&mut reader);

    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();
    let open_d: OpenD = smem.read_value(0).unwrap();

    let expected = test_open_d();
    assert_eq!(open_d, expected);

    send_quit(&mut child.stdin.take().unwrap());
    let status = child.wait().unwrap();
    assert!(status.success());
}

// ---- Test: Rust writes OpenD, C# reads ----

#[test]
fn test_rust_writes_open_d_csharp_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_RustWritesOpenD";

    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();
    let open_d = test_open_d();
    smem.write_value(0, &open_d).unwrap();

    let (code, stdout, stderr) = run_csharp_and_wait(&["verify-open-d", mmf_name]);

    eprintln!("C# stdout: {stdout}");
    if !stderr.is_empty() {
        eprintln!("C# stderr: {stderr}");
    }
    assert_eq!(code, 0, "C# verification failed");
    assert!(stdout.contains("OK"));

    drop(smem);
}

// ---- Test: C# writes Panel array, Rust reads ----

#[test]
fn test_csharp_writes_panel_rust_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_CSharpWritesPanel";
    let mut child = run_csharp(&["write-panel", mmf_name]);

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    wait_for_ready(&mut reader);

    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();

    // Read length prefix
    let length: i32 = smem.read_value(0).unwrap();
    assert_eq!(length, 256);

    // Read array data
    let data: Vec<i32> = smem.read_array(4, length as usize).unwrap();
    assert_eq!(data, test_panel());

    send_quit(&mut child.stdin.take().unwrap());
    let status = child.wait().unwrap();
    assert!(status.success());
}

// ---- Test: Rust writes Panel array, C# reads ----

#[test]
fn test_rust_writes_panel_csharp_reads() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let mmf_name = "InteropTest_RustWritesPanel";

    let smem = bids_smemlib::smem::windows::WindowsSMemIF::open_or_create(mmf_name, 4096).unwrap();
    let panel = test_panel();

    // Write length prefix + array data (matching ArrayDataSMemCtrler layout)
    let length = panel.len() as i32;
    smem.write_value(0, &length).unwrap();
    smem.write_array(4, &panel).unwrap();

    let (code, stdout, stderr) = run_csharp_and_wait(&["verify-panel", mmf_name]);

    eprintln!("C# stdout: {stdout}");
    if !stderr.is_empty() {
        eprintln!("C# stderr: {stderr}");
    }
    assert_eq!(code, 0, "C# verification failed");
    assert!(stdout.contains("OK"));

    drop(smem);
}

// ---- Test: Layout dump (for debugging) ----

#[test]
fn test_layout_dump() {
    assert!(build_csharp(), "Failed to build C# interop test project");

    let (code, stdout, stderr) = run_csharp_and_wait(&["dump-layout"]);

    eprintln!("C# layout dump:\n{stdout}");
    if !stderr.is_empty() {
        eprintln!("C# stderr: {stderr}");
    }
    assert_eq!(code, 0, "dump-layout failed");

    // Verify C# sizes match Rust sizes
    assert!(stdout.contains("sizeof(Spec) = 20"), "Spec size mismatch");
    assert!(stdout.contains("sizeof(State) = 40"), "State size mismatch");
    assert!(stdout.contains("sizeof(Hand) = 16"), "Hand size mismatch");
    assert!(stdout.contains("sizeof(Hands) = 32"), "Hands size mismatch");
    assert!(
        stdout.contains("sizeof(PreTrainD) = 32"),
        "PreTrainD size mismatch"
    );
    assert!(stdout.contains("sizeof(OpenD) = 80"), "OpenD size mismatch");
    assert!(
        stdout.contains("sizeof(BIDSSharedMemoryData) = 96"),
        "BSMD size mismatch"
    );
}
