use sea_orm::DatabaseConnection;

pub mod model;
pub mod repository;
pub mod rpc;
pub mod v1;

pub struct HypervisorsService {
    database_connection: DatabaseConnection,
}
