pub trait ExtractToken {
    fn access_token(&self) -> Option<String>;
}

impl<T> ExtractToken for tonic::Request<T> {
    fn access_token(&self) -> Option<String> {
        self.metadata()
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer"))
            .map(|value| value.to_owned())
    }
}
