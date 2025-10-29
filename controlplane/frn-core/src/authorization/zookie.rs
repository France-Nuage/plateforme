use spicedb::api::v1::ZedToken;

pub struct Zookie {
    pub token: String,
}

impl From<ZedToken> for Zookie {
    fn from(value: ZedToken) -> Self {
        Zookie { token: value.token }
    }
}
