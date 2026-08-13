use std::sync::Arc;

use spicedb::SpiceDB;
use workflow::WorkerContext;

pub fn context_with_spicedb(spicedb: SpiceDB) -> WorkerContext {
    let pool = sqlx::PgPool::connect_lazy("postgres://localhost:1/unused")
        .expect("could not build lazy pool");
    let kube = kube::Client::try_from(kube::Config::new(
        "https://127.0.0.1:1".parse().expect("valid kube api uri"),
    ))
    .expect("could not build kube client");

    WorkerContext {
        pool,
        spicedb,
        kube,
        platform_config: workflow::PlatformConfig {
            default_storage_class: None,
            cnpg_backup_enabled: false,
            deployment_labels: std::collections::BTreeMap::new(),
            deployment_annotations: std::collections::BTreeMap::new(),
        },
        kek: Arc::new(frn_crypto::Kek::from_bytes([42u8; 32])),
        kubeconfig_path: None,
    }
}
