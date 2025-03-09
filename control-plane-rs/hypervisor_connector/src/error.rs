#[derive(Debug)]
pub enum Error {
    Unauthorized(Box<dyn std::error::Error>),
    Network(Box<dyn std::error::Error>),
    Other,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        println!("Error: {:?}", err);
        match err.status() {
            Some(status) => match status {
                reqwest::StatusCode::UNAUTHORIZED => Error::Unauthorized(Box::new(err)),
                _ => Error::Network(Box::new(err)),
            },
            _ => Error::Other,
        }
    }
}
