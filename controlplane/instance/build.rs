fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .extern_path(
            ".francenuage.fr.api.controlplane.v1.problem",
            "::problem::v1",
        )
        .compile_protos(&["instances.proto"], &[".", ".."])?;

    Ok(())
}
