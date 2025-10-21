use crate::identity::{ServiceAccount, User};

pub enum Principal {
    ServiceAccount(ServiceAccount),
    User(User),
}
