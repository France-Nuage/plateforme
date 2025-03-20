fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("controlplane.v0.proto")?;
    Ok(())
}
