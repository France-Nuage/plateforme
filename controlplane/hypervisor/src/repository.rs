use uuid::Uuid;

use crate::model;

pub struct Repository {}

impl Repository {
    fn foo() {
        let hypervisor = model::ActiveModel {
            id: sea_orm::ActiveValue::Set(Uuid::new_v4().to_string()),
        };
    }
}
