use errors::ClientError;
use models::Class;

pub mod errors;
pub mod holi_yoga;
pub mod models;
pub mod plastilin;

pub trait ClassCRUD {
    async fn list_user_classes(&self) -> Result<Vec<Class>, ClientError>;
}
