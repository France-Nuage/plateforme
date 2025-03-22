fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Download the validate.proto file if needed
    if !std::path::Path::new("validate/validate.proto").exists() {
        std::fs::create_dir_all("validate")?;
        let validate_proto = reqwest::blocking::get(
            "https://raw.githubusercontent.com/bufbuild/protoc-gen-validate/main/validate/validate.proto",
        )?
        .text()?;
        std::fs::write("validate/validate.proto", validate_proto)?;
    }

    tonic_build::compile_protos("controlplane.proto")?;
    Ok(())
}
