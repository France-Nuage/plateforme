use auth::Authorize as Foo; // Re-exported from database-core
use derive_auth::Authorize;
use uuid::Uuid;

#[derive(Authorize)]
struct Anvil {
    id: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anvil_authorize() {
        let anvil_id = Uuid::new_v4();
        let anvil = Anvil { id: anvil_id };

        let (resource_type, id_ref) = anvil.resource();

        assert_eq!(resource_type, "anvil");
        assert_eq!(id_ref, &anvil_id);
    }
}
