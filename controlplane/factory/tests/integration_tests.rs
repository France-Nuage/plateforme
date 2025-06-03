//! Integration tests for the Factory derive macro
//!
//! These tests verify that the macro generates working code
//! and serves as documentation examples.

use derive_factory::Factory;

#[derive(Factory, Debug, PartialEq)]
struct Missile {
    #[factory(relation = "CategoryFactory")]
    category_id: String,
    max_range: u32,
    owner: String,
    target: String,
}

#[derive(Factory)]
struct Category {
    #[allow(dead_code)]
    name: String,
}

#[tokio::test]
async fn test_a_struct_can_be_factorized() {
    let missile = Missile::factory()
        .owner("Wile E. Coyote".to_string())
        .target("Road Runner".to_owned())
        .create()
        .await
        .expect("Should create successfully");

    assert_eq!(
        missile,
        Missile {
            category_id: String::default(),
            max_range: u32::default(),
            owner: "Wile E. Coyote".to_owned(),
            target: "Road Runner".to_owned()
        }
    );
}

#[tokio::test]
async fn test_a_struct_can_be_factorized_with_a_relation() {
    let missile = Missile::factory()
        .for_category_with(|category| category.name("Weaponry".into()))
        .owner("Wile E. Coyote".to_string())
        .target("Road Runner".to_owned())
        .create()
        .await
        .expect("Should create successfully");

    assert_eq!(
        missile,
        Missile {
            category_id: String::default(),
            max_range: u32::default(),
            owner: "Wile E. Coyote".to_owned(),
            target: "Road Runner".to_owned()
        }
    );
}
