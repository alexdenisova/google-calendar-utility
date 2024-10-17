use errors::ClientError;
use models::{Class, UtcDateTime};

pub mod errors;
pub mod holi_yoga;
pub mod models;
pub mod plastilin;

pub trait ClassCRUD {
    fn name() -> String;
    async fn get_user_classes(&self) -> Result<Vec<Class>, ClientError>;
    async fn list_day_classes(&self, day: &UtcDateTime) -> Result<Vec<Class>, ClientError>;
    async fn sign_up_for_class(&self, class: &Class) -> Result<(), ClientError>;
}
