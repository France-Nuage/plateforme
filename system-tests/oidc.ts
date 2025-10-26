import { User } from "@/types";
import { faker } from "@faker-js/faker";

type RegistrationResponse = {
  id_token: string;
  session_state: null;
  access_token: string;
  token_type: 'Bearer';
  scope: 'email openid profile';
  profile: {
    aud: 'francenuage';
    exp: number;
    iat: number;
    iss: 'https://oidc';
  };
  expires_at: number;
};

/**
 * Obtains an admin access token from Keycloak for administrative operations.
 *
 * @returns Admin access token
 */
async function getAdminToken(): Promise<string> {
  const keycloakUrl = process.env.KEYCLOAK_URL || 'https://keycloak.test';
  const adminUsername = process.env.KEYCLOAK_ADMIN || 'admin';
  const adminPassword = process.env.KEYCLOAK_ADMIN_PASSWORD || 'admin';

  const response = await fetch(`${keycloakUrl}/realms/master/protocol/openid-connect/token`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: new URLSearchParams({
      grant_type: 'password',
      client_id: 'admin-cli',
      username: adminUsername,
      password: adminPassword,
    }),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Failed to obtain admin token: ${response.status} ${error}`);
  }

  const data = await response.json();


  console.log('obtained admin token:', data);

  return data.access_token;
}

/**
 * Creates a new user in Keycloak dynamically using the Admin API.
 *
 * This function:
 * 1. Generates user data from provided partial user or uses faker for missing fields
 * 2. Obtains an admin access token from Keycloak
 * 3. Creates the user in the francenuage realm via Keycloak Admin API
 * 4. Obtains user tokens via direct access grant (password flow)
 * 5. Returns tokens in the expected RegistrationResponse format for test fixtures
 *
 * @param user - Partial user data. Missing fields are auto-generated.
 * @returns Promise resolving to RegistrationResponse with user tokens
 */
export const createUser = async (user: Partial<User>): Promise<RegistrationResponse> => {
  const keycloakUrl = process.env.KEYCLOAK_URL || 'https://keycloak.test';
  const realmName = 'francenuage';

  // Generate user data with defaults
  const userData = {
    username: user.username ?? faker.internet.username(),
    email: user.email ?? faker.internet.email(),
    firstName: user.name?.split(' ')[0] ?? faker.person.firstName(),
    lastName: user.name?.split(' ').slice(1).join(' ') || faker.person.lastName(),
    password: user.password ?? faker.internet.password(),
  };

  // Get admin token
  const adminToken = await getAdminToken();

  // Create user in Keycloak
  const createUserResponse = await fetch(`${keycloakUrl}/admin/realms/${realmName}/users`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${adminToken}`,
    },
    body: JSON.stringify({
      username: userData.username,
      email: userData.email,
      firstName: userData.firstName,
      lastName: userData.lastName,
      enabled: true,
      emailVerified: true,
      credentials: [
        {
          type: 'password',
          value: userData.password,
          temporary: false,
        },
      ],
    }),
  });

  if (!createUserResponse.ok) {
    const error = await createUserResponse.text();
    throw new Error(`Failed to create user in Keycloak: ${createUserResponse.status} ${error}`);
  }

  // Get user tokens using direct access grant (Resource Owner Password Credentials flow)
  const tokenResponse = await fetch(`${process.env.OIDC_PROVIDER_URL}/protocol/openid-connect/token`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
    },
    body: new URLSearchParams({
      grant_type: 'password',
      client_id: process.env.OIDC_CLIENT_ID || 'francenuage',
      username: userData.username,
      password: userData.password,
      scope: 'openid profile email',
    }),
  });

  if (!tokenResponse.ok) {
    const error = await tokenResponse.text();
    throw new Error(`Failed to obtain user tokens: ${tokenResponse.status} ${error}`);
  }

  const tokens = await tokenResponse.json();

  // Parse the ID token to extract profile information
  const idTokenPayload = JSON.parse(atob(tokens.id_token.split('.')[1]));

  console.log('obtained token for created user:', idTokenPayload);

  // Return in the expected RegistrationResponse format
  return {
    id_token: tokens.id_token,
    session_state: null,
    access_token: tokens.access_token,
    token_type: 'Bearer',
    scope: 'email openid profile',
    profile: {
      aud: 'francenuage',
      exp: idTokenPayload.exp,
      iat: idTokenPayload.iat,
      iss: idTokenPayload.iss,
    },
    expires_at: Date.now() + (tokens.expires_in * 1000),
  };
}

function initializeContext() {
}
