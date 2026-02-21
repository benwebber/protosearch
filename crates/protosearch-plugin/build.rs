use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let proto_dir = PathBuf::from("../../proto");

    println!("cargo:rerun-if-changed={}", proto_dir.to_str().unwrap());

    // Use protoc directly through a Command to generate a descriptor set.
    // This is more reliable than fighting with protobuf-codegen's wrapper
    // which insists on generating .rs files.
    let protoc = protoc_bin_vendored::protoc_bin_path().unwrap();
    let status = std::process::Command::new(protoc)
        .arg("-I")
        .arg(&proto_dir)
        .arg("--include_imports")
        .arg("--include_source_info")
        .arg("--descriptor_set_out")
        .arg(out_dir.join("tests.pb"))
        .arg(proto_dir.join("tests/tests.proto"))
        .status()
        .expect("failed to execute protoc");

    if !status.success() {
        panic!("protoc failed with status {}", status);
    }
}
