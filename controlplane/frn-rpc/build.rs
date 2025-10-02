fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("resources.proto").map_err(Into::into)
}
