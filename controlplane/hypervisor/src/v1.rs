tonic::include_proto!("francenuage.fr.api.controlplane.v1.hypervisors");

impl From<Hypervisor> for crate::model::ActiveModel {
    fn from(value: Hypervisor) -> Self {
        crate::model::ActiveModel {
            id: sea_orm::ActiveValue::Set(uuid::Uuid::new_v4().to_string()),
        }
    }
}
