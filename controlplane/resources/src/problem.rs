use thiserror::Error;

#[derive(Debug, Error)]
pub enum Problem {
    #[error("Other: {0}")]
    Other(Box<dyn std::error::Error + Send + Sync>),
}

/// Converts a `resources::projects::Problem` into a `resources::Problem`.
impl From<crate::projects::Problem> for Problem {
    fn from(problem: crate::projects::Problem) -> Self {
        Problem::Other(Box::new(problem))
    }
}
