pub enum Error {}

impl Error {
    pub fn convert(error: frn_core::Error) -> tonic::Status {
        Error::from(error).into()
    }
    pub fn sqlx_to_status(error: sqlx::Error) -> tonic::Status {
        Error::from(error).into()
    }
}

impl From<sqlx::Error> for Error {
    fn from(_value: sqlx::Error) -> Self {
        todo!()
    }
}

impl From<frn_core::Error> for Error {
    fn from(_value: frn_core::Error) -> Self {
        todo!()
    }
}

impl From<Error> for tonic::Status {
    fn from(_value: Error) -> Self {
        todo!()
    }
}
