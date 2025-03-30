#[derive(Debug)]
pub enum Problem {
    InstanceNotFound {
        id: String,
        source: Box<dyn std::error::Error>,
    },
    Other(Box<dyn std::error::Error>),
}
