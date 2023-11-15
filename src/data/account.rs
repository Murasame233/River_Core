use uuid::Uuid;

#[derive(Debug,Clone)]
pub struct Account {
    pub address: String,
    pub father: Option<Uuid>,
    pub sub: Option<Vec<Uuid>>,
}
