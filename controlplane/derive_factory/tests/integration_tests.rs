//! Integration tests for the Factory derive macro
//!
//! These tests verify that the macro generates working code
//! and serves as documentation examples.

use database::Persistable;
use derive_factory::Factory;

#[derive(Debug, Default, Factory, PartialEq)]
struct Missile {
    #[factory(relation = "CategoryFactory")]
    category_id: String,
    max_range: u32,
    owner: String,
    target: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl database::Persistable for Missile {
    type Connection = ();
    type Error = ();

    async fn create(self, _pool: Self::Connection) -> Result<Self, Self::Error> {
        Ok(self)
    }

    async fn update(self, _pool: Self::Connection) -> Result<Self, Self::Error> {
        Ok(self)
    }
}

#[derive(Debug, Default, Factory)]
struct Category {
    id: String,
    #[allow(dead_code)]
    name: String,
}

impl database::Persistable for Category {
    type Connection = ();
    type Error = ();

    async fn create(self, _pool: ()) -> Result<Self, Self::Error> {
        Ok(self)
    }

    async fn update(self, _pool: ()) -> Result<Self, Self::Error> {
        Ok(self)
    }
}

#[tokio::test]
async fn test_a_struct_can_be_factorized() {
    let missile = Missile::factory()
        .owner("Wile E. Coyote".to_string())
        .target("Road Runner".to_owned())
        .create(())
        .await
        .expect("Should create successfully");

    assert_eq!(
        missile,
        Missile {
            category_id: String::default(),
            max_range: u32::default(),
            owner: "Wile E. Coyote".to_owned(),
            target: "Road Runner".to_owned(),
            created_at: chrono::DateTime::default()
        }
    );
}

// #[tokio::test]
// async fn test_a_struct_can_be_factorized_with_a_relation() {
//     let missile = Missile::factory::<()>()
//         .for_category_with(|category| category.name("Weaponry".into()))
//         .owner("Wile E. Coyote".to_string())
//         .target("Road Runner".to_owned())
//         .create(())
//         .await
//         .expect("Should create successfully");
//
//     assert_eq!(
//         missile,
//         Missile {
//             category_id: String::default(),
//             max_range: u32::default(),
//             owner: "Wile E. Coyote".to_owned(),
//             target: "Road Runner".to_owned(),
//             created_at: chrono::DateTime::default()
//         }
//     );
// }
