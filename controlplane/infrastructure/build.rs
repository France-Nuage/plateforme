fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("infrastructure.proto")
        .map(|_| ())
        .map_err(Into::into)
}
