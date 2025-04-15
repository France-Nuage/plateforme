use sea_orm::ActiveValue;
use uuid::Uuid;

tonic::include_proto!("francenuage.fr.api.controlplane.v1.hypervisors");

impl From<RegisterHypervisorRequest> for crate::model::ActiveModel {
    fn from(value: RegisterHypervisorRequest) -> Self {
        crate::model::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4().to_string()),
            url: ActiveValue::Set(value.url),
            authentication_token: ActiveValue::Set(value.authentication_token),
            storage_name: ActiveValue::Set(value.storage_name),
        }
    }
}

impl From<crate::model::Model> for Hypervisor {
    fn from(value: crate::model::Model) -> Self {
        Hypervisor { url: value.url }
    }
}
