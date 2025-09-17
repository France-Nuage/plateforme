fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::configure()
        .build_server(true)
        .compile_protos(
            &[
                "proto/authzed/api/v1/core.proto",
                "proto/authzed/api/v1/permission_service.proto",
                "proto/authzed/api/v1/schema_service.proto",
                "proto/authzed/api/v1/watch_service.proto",
                "proto/authzed/api/v1/experimental_service.proto",
                "proto/authzed/api/v1/debug.proto",
                "proto/authzed/api/v1/error_reason.proto",
            ],
            &["proto"],
        )
        .map_err(Into::into)
}
