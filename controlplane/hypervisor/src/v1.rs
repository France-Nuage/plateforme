use sea_orm::ActiveValue;
use uuid::Uuid;

tonic::include_proto!("francenuage.fr.api.controlplane.v1.hypervisors");

impl From<Hypervisor> for crate::model::ActiveModel {
    fn from(value: Hypervisor) -> Self {
        crate::model::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4().to_string()),
            url: ActiveValue::Set(value.url),
        }
    }
}

impl From<crate::model::Model> for Hypervisor {
    fn from(value: crate::model::Model) -> Self {
        Hypervisor { url: value.url }
    }
}
