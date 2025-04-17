use std::{ops::Deref, sync::Arc};

use sea_orm::{DatabaseConnection, DbErr, EntityTrait};

use crate::model::{Entity, Model};

pub struct HypervisorsService {
    db: Arc<DatabaseConnection>,
}

impl HypervisorsService {
    pub async fn list(&self) -> Result<Vec<Model>, DbErr> {
        Entity::find().all(self.db.deref()).await
    }

    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self { db }
    }
}
