fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Compile proto files and generate file descriptor set
    let out_dir = std::env::var("OUT_DIR")?;
    let descriptor_path = std::path::PathBuf::from(&out_dir).join("descriptor.bin");

    tonic_prost_build::configure()
        .file_descriptor_set_path(&descriptor_path)
        .compile_protos(
            &["compute.proto", "iam.proto", "resourcemanager.proto"],
            &["."],
        )?;

    // Read the descriptor set and write it for both v1 and v1alpha reflection services
    let descriptor_set = std::fs::read(&descriptor_path)?;

    // Write v1 reflection descriptor
    let reflection_v1_path =
        std::path::PathBuf::from(&out_dir).join("reflection_descriptor_v1.bin");
    std::fs::write(reflection_v1_path, &descriptor_set)?;

    // Write v1alpha reflection descriptor (same content, different service version)
    let reflection_v1alpha_path =
        std::path::PathBuf::from(&out_dir).join("reflection_descriptor_v1alpha.bin");
    std::fs::write(reflection_v1alpha_path, descriptor_set)?;

    Ok(())
}
