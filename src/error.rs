use thiserror::Error;

#[derive(Error, Debug)]
pub enum OliveError {
    #[error("Graph error: {0}")]
    Graph(String),

    #[error("Cycle detected in graph: {0}")]
    CycleDetected(String),

    #[error("Pin not found: {0}")]
    PinNotFound(String),

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Incompatible pin types: {0} -> {1}")]
    IncompatibleTypes(String, String),

    #[error("Timeline error: {0}")]
    Timeline(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Render error: {0}")]
    Render(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, OliveError>;
