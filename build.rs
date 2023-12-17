fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/network.proto");
    let _ = protobuf_codegen::Codegen::new()
        .pure()
        .includes(["src/"])
        .input("src/proto/network.proto")
        .out_dir("src/proto")
        .run();
    Ok(())
}
