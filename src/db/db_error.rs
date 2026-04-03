#[derive(Debug)]
pub enum DbErrors{
    FailedToInitializeDatabase(String),
    DbFailedToRespond(String), 
}