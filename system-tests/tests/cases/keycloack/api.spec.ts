import { test, expect } from "../../base";

test.describe('Keycloak', () => {
  test('I can generate an admin token through the http api', async ({ keycloak }) => {
    const response = await keycloak.getAdminToken();

    expect(response).toEqual(expect.objectContaining({
      access_token: expect.any(String),
      expires_in: expect.any(Number),
      refresh_expires_in: expect.any(Number),
      refresh_token: expect.any(String),
      token_type: expect.any(String),
      'not-before-policy': expect.any(Number),
      session_state: expect.any(String),
      scope: expect.any(String)
    }));
  });

  test('I can create a user through the http api', async ({ keycloak }) => {
    const response = await keycloak.createUser();

    expect(response).toEqual(expect.objectContaining({
      access_token: expect.any(String),
      expires_in: expect.any(Number),
      refresh_expires_in: expect.any(Number),
      refresh_token: expect.any(String),
      token_type: expect.any(String),
      'not-before-policy': expect.any(Number),
      session_state: expect.any(String),
      scope: expect.any(String)
    }));
  })
});

