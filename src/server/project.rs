use uuid::Uuid;

/// A project submitted to the server
pub struct Project {
    uuid: Uuid,
    name: String,
}
