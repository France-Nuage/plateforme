fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .compile_protos(&["compute.proto", "resourcemanager.proto"], &["."])
        .map_err(Into::into)
}
