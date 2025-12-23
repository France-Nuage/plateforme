use crate::Error;
use crate::authorization::{Authorize, Permission, Principal, Relation, Relationship, Resource};
use crate::identity::{ServiceAccount, User};
use crate::resourcemanager::{DEFAULT_PROJECT_NAME, Project};
// use database::{Factory, Persistable, Repository};
use fabrique::{Factory, Persistable};
use sqlx::types::chrono;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

/// Maximum length for a DNS subdomain label
const MAX_SLUG_LENGTH: usize = 63;

#[derive(Debug, Default, Factory, Persistable, Resource)]
pub struct Organization {
    /// The organization id
    #[fabrique(primary_key)]
    pub id: Uuid,
    /// The organization name
    pub name: String,
    /// The organization slug (DNS-compatible identifier)
    pub slug: String,
    /// The organization parent, if any
    pub parent_id: Option<Uuid>,
    /// Creation time of the organization
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update time of the organization
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Generate a DNS-compatible slug from a name.
///
/// The slug follows RFC 1123 subdomain rules:
/// - Only lowercase alphanumeric characters and hyphens
/// - Cannot start or end with a hyphen
/// - Maximum 63 characters
/// - Cannot be empty
fn generate_slug(name: &str) -> Result<String, Error> {
    // Transliterate accented characters and normalize
    let normalized: String = name.chars().map(transliterate_char).collect();

    // Convert to lowercase, replace spaces/underscores with hyphens
    let slug: String = normalized
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '_' || c == '-' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|&c| c != '\0')
        .collect();

    // Collapse multiple hyphens and trim leading/trailing hyphens
    let slug: String = slug
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    // Truncate to max length (at hyphen boundary if possible)
    let slug = truncate_slug(&slug, MAX_SLUG_LENGTH);

    if slug.is_empty() {
        return Err(Error::InvalidSlug("slug cannot be empty".to_string()));
    }

    Ok(slug)
}

/// Transliterate common accented characters to ASCII equivalents
fn transliterate_char(c: char) -> char {
    match c {
        'à' | 'â' | 'ä' | 'á' | 'ã' | 'À' | 'Â' | 'Ä' | 'Á' | 'Ã' => 'a',
        'è' | 'ê' | 'ë' | 'é' | 'È' | 'Ê' | 'Ë' | 'É' => 'e',
        'ì' | 'î' | 'ï' | 'í' | 'Ì' | 'Î' | 'Ï' | 'Í' => 'i',
        'ò' | 'ô' | 'ö' | 'ó' | 'õ' | 'Ò' | 'Ô' | 'Ö' | 'Ó' | 'Õ' => 'o',
        'ù' | 'û' | 'ü' | 'ú' | 'Ù' | 'Û' | 'Ü' | 'Ú' => 'u',
        'ç' | 'Ç' => 'c',
        'ñ' | 'Ñ' => 'n',
        'ß' => 's',
        'æ' | 'Æ' => 'a',
        'œ' | 'Œ' => 'o',
        _ => c,
    }
}

/// Truncate slug to max length, preferring to break at hyphen boundaries
fn truncate_slug(slug: &str, max_len: usize) -> String {
    if slug.len() <= max_len {
        return slug.to_string();
    }

    let truncated = &slug[..max_len];
    // Try to break at last hyphen to avoid cutting words
    if let Some(last_hyphen) = truncated.rfind('-') {
        if last_hyphen > max_len / 2 {
            return truncated[..last_hyphen].to_string();
        }
    }
    truncated.trim_end_matches('-').to_string()
}

#[derive(Clone)]
pub struct Organizations<A: Authorize> {
    auth: A,
    db: Pool<Postgres>,
}

impl<A: Authorize> Organizations<A> {
    pub fn new(auth: A, db: Pool<Postgres>) -> Self {
        Self { auth, db }
    }

    pub async fn list<P: Principal>(&mut self, principal: &P) -> Result<Vec<Organization>, Error> {
        self.auth
            .lookup::<Organization>()
            .on_behalf_of(principal)
            .with(Permission::Get)
            .against(&self.db)
            .await
    }

    pub async fn create_organization<P: Principal + Sync>(
        &mut self,
        connection: &Pool<Postgres>,
        _principal: &P,
        name: String,
        parent_id: Option<Uuid>,
    ) -> Result<Organization, Error> {
        // self.auth
        //     .can(principal)
        //     .perform(Permission::Create)
        //     .over(&Organization::any())
        //     .await?;

        tracing::info!(
            "received request to create organization with name '{}' and parent id '{:?}'",
            &name,
            &parent_id
        );

        // Generate slug from name
        let slug = generate_slug(&name)?;

        // Check for slug collision
        let existing: Option<(Uuid,)> =
            sqlx::query_as("SELECT id FROM organizations WHERE slug = $1")
                .bind(&slug)
                .fetch_optional(connection)
                .await?;

        if existing.is_some() {
            return Err(Error::SlugAlreadyExists(slug));
        }

        // Create the organization
        let organization = Organization::factory()
            .id(Uuid::new_v4())
            .name(name)
            .slug(slug)
            .parent_id(parent_id)
            .create(connection)
            .await?;

        // Create the parent relationship if specified
        if let Some(parent_id) = parent_id {
            let parent: Organization =
                sqlx::query_as("SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE id = $1")
                    .bind(parent_id)
                    .fetch_one(&self.db)
                    .await?;

            Relationship::new(&parent, Relation::Parent, &organization)
                .publish(&self.db)
                .await?;
        }

        let project = Project::factory()
            .id(Uuid::new_v4())
            .name(DEFAULT_PROJECT_NAME.to_owned())
            .organization_id(organization.id)
            .create(&self.db)
            .await?;

        Relationship::new(&organization, Relation::Parent, &project)
            .publish(&self.db)
            .await?;

        Ok(organization)
    }

    pub async fn add_service_account(
        &mut self,
        organization: &Organization,
        service_account: &ServiceAccount,
    ) -> Result<(), Error> {
        // Create the associated in the relational database
        sqlx::query!("INSERT INTO organization_service_account(organization_id, service_account_id) VALUES ($1, $2) ON CONFLICT (organization_id, service_account_id) DO NOTHING", organization.id(), service_account.id()).execute(&self.db).await?;

        // Create the relation for dispatch in the authorization database
        Relationship::new(service_account, Relation::Member, organization)
            .publish(&self.db)
            .await?;

        Ok(())
    }

    pub async fn add_user(
        &mut self,
        organization: &Organization,
        user: &User,
    ) -> Result<(), Error> {
        // Create the associated in the relational database
        sqlx::query!("INSERT INTO organization_user(organization_id, user_id) VALUES ($1, $2) ON CONFLICT (organization_id, user_id) DO NOTHING", organization.id(), user.id()).execute(&self.db).await?;

        // Create the relation for dispatch in the authorization database
        Relationship::new(user, Relation::Member, organization)
            .publish(&self.db)
            .await?;

        Ok(())
    }

    pub async fn initialize_root_organization(
        &mut self,
        organization_name: String,
    ) -> Result<Organization, Error> {
        // Attempt to retrieve the organization from the database
        let maybe_organization: Option<Organization> =
            sqlx::query_as("SELECT id, name, slug, parent_id, created_at, updated_at FROM organizations WHERE name = $1 LIMIT 1")
                .bind(&organization_name)
                .fetch_optional(&self.db)
                .await?;

        // Create the root organization if there is no database match
        let organization = match maybe_organization {
            Some(organization) => organization,
            None => {
                let slug = generate_slug(&organization_name)?;
                Organization::factory()
                    .name(organization_name)
                    .slug(slug)
                    .create(&self.db)
                    .await?
            }
        };

        // Create the default project for the root organization
        sqlx::query!(
            r#"
            INSERT INTO projects (name, organization_id) 
            SELECT 'unattributed', $1
            WHERE NOT EXISTS (
                SELECT 1 FROM projects 
                WHERE name = 'unattributed' AND organization_id = $1
            )
            "#,
            &organization.id
        )
        .execute(&self.db)
        .await?;

        Ok(organization)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_basic() {
        assert_eq!(
            generate_slug("Mon Organisation").unwrap(),
            "mon-organisation"
        );
        assert_eq!(generate_slug("Test Company").unwrap(), "test-company");
    }

    #[test]
    fn test_generate_slug_accented_characters() {
        assert_eq!(generate_slug("Café Français").unwrap(), "cafe-francais");
        assert_eq!(generate_slug("Ñoño España").unwrap(), "nono-espana");
        // ß is transliterated to single 's' (char-to-char mapping limitation)
        assert_eq!(generate_slug("Größe").unwrap(), "grose");
        assert_eq!(
            generate_slug("Ça va être génial").unwrap(),
            "ca-va-etre-genial"
        );
    }

    #[test]
    fn test_generate_slug_special_characters() {
        assert_eq!(generate_slug("Test@Company!").unwrap(), "testcompany");
        assert_eq!(generate_slug("Hello, World!").unwrap(), "hello-world");
        assert_eq!(generate_slug("Test_Underscore").unwrap(), "test-underscore");
    }

    #[test]
    fn test_generate_slug_leading_trailing_hyphens() {
        assert_eq!(generate_slug("--Test--").unwrap(), "test");
        assert_eq!(generate_slug("---Leading").unwrap(), "leading");
        assert_eq!(generate_slug("Trailing---").unwrap(), "trailing");
        assert_eq!(generate_slug("--Both--Sides--").unwrap(), "both-sides");
    }

    #[test]
    fn test_generate_slug_multiple_spaces_hyphens() {
        assert_eq!(
            generate_slug("Multiple   Spaces").unwrap(),
            "multiple-spaces"
        );
        assert_eq!(
            generate_slug("Multiple---Hyphens").unwrap(),
            "multiple-hyphens"
        );
        assert_eq!(
            generate_slug("Mixed - - Separators").unwrap(),
            "mixed-separators"
        );
    }

    #[test]
    fn test_generate_slug_numbers() {
        assert_eq!(generate_slug("123 Company").unwrap(), "123-company");
        assert_eq!(generate_slug("Company 456").unwrap(), "company-456");
        assert_eq!(generate_slug("123").unwrap(), "123");
    }

    #[test]
    fn test_generate_slug_single_character() {
        assert_eq!(generate_slug("A").unwrap(), "a");
        assert_eq!(generate_slug("1").unwrap(), "1");
    }

    #[test]
    fn test_generate_slug_empty_result() {
        assert!(generate_slug("").is_err());
        assert!(generate_slug("---").is_err());
        assert!(generate_slug("@#$%").is_err());
        assert!(generate_slug("   ").is_err());
    }

    #[test]
    fn test_generate_slug_max_length() {
        let long_name = "a".repeat(100);
        let result = generate_slug(&long_name).unwrap();
        assert!(result.len() <= 63);
        assert_eq!(result.len(), 63);
    }

    #[test]
    fn test_generate_slug_truncation_at_hyphen() {
        // Create a name that will result in a slug > 63 chars with hyphens
        let long_name = "this-is-a-very-long-organization-name-that-should-be-truncated-at-hyphen";
        let result = generate_slug(long_name).unwrap();
        assert!(result.len() <= 63);
        assert!(!result.ends_with('-'));
    }

    #[test]
    fn test_generate_slug_dns_compatible() {
        // Verify DNS compatibility: only lowercase alphanumeric and hyphens
        let test_cases = vec![
            "Mon Organisation",
            "Café Français",
            "Test@Company!",
            "123 Numbers",
        ];

        for name in test_cases {
            let slug = generate_slug(name).unwrap();
            // Must not start with hyphen
            assert!(!slug.starts_with('-'), "Slug '{}' starts with hyphen", slug);
            // Must not end with hyphen
            assert!(!slug.ends_with('-'), "Slug '{}' ends with hyphen", slug);
            // Must only contain allowed characters
            assert!(
                slug.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "Slug '{}' contains invalid characters",
                slug
            );
            // Must not exceed 63 characters
            assert!(slug.len() <= 63, "Slug '{}' exceeds 63 characters", slug);
        }
    }
}
