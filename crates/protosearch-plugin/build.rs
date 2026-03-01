use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not defined"));
    let proto_dir = PathBuf::from("../../proto");
    let protoc = protoc_bin_vendored::protoc_bin_path().expect("cannot find bundled protoc");

    println!("cargo:rerun-if-changed={}", proto_dir.display());

    protobuf_codegen::Codegen::new()
        .protoc()
        .protoc_path(&protoc)
        .include(&proto_dir)
        .input(proto_dir.join("protosearch/protosearch.proto"))
        .cargo_out_dir("proto")
        .run_from_script();

    // The generated code includes inner attributes that can't be used inside `include!`.
    // We either need to strip the attributes or output the file to `src/` directly.
    // <https://github.com/rust-lang/rust/issues/117464>
    let generated = std::fs::read_to_string(out_dir.join("proto/protosearch.rs"))
        .expect("cannot read generated protosearch module");
    let stripped: String = generated
        .lines()
        .filter(|line| !line.starts_with("#![") && !line.starts_with("//!"))
        .flat_map(|line| [line, "\n"])
        .collect();
    std::fs::write(out_dir.join("protosearch.rs"), stripped)
        .expect("failed to write to standard output");
}
