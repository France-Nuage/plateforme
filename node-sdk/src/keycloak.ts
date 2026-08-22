import { user } from './fixtures/user';
import { User } from './models';
import { TokenResponse, UserInfoResponse } from './types';

export class KeyCloakApi {
  /** The Keycloak admin credentials. */
  public admin: {
    username: string;
    password: string;
  };

  /** The Keycloak default realm name. */
  public realm: string;

  /** The Keycloak api url. */
  public url: string;

  /** The class constructor. */
  public constructor(
    url: string,
    admin: { username: string; password: string },
  ) {
    this.admin = admin;
    this.realm = 'francenuage';
    this.url = url;
  }

  /**
   * Create a user (idempotently) and return a token for it.
   *
   * Most callers pass a randomly generated identity, but some use a fixed email
   * (e.g. the platform admin the payment E2E signs in as). When the environment
   * is reused across runs — its Keycloak database persists on a PVC — that fixed
   * user already exists, and Keycloak answers the creation with `409 Conflict`.
   * We treat that as success: the user is present with the same fixed password,
   * so we skip straight to fetching its token instead of failing the whole run.
   */
  public async createUser(
    data?: Partial<User>,
    realm?: string,
  ): Promise<TokenResponse> {
    const token = (await this.getAdminToken()).access_token;
    const newUser = { ...user({ password: 'password' }), ...data };

    if (!newUser.username) {
      throw new Error('missing username for authentication');
    }

    if (!newUser.password) {
      throw new Error('missing password for authentication');
    }

    const response = await fetch(
      `${this.url}/admin/realms/${realm ?? this.realm}/users`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({
          username: newUser.username,
          email: newUser.email,
          firstName: newUser.firstName,
          lastName: newUser.lastName,
          enabled: true,
          emailVerified: true,
          credentials: [
            {
              type: 'password',
              value: newUser.password,
              temporary: false,
            },
          ],
        }),
      },
    );

    // 409 Conflict => the user already exists (reused environment). That is not
    // an error for us: fall through to fetch its token below.
    if (!response.ok && response.status !== 409) {
      throw new Error(`could not create user -- ${await response.text()}`);
    }

    return await this.getUserToken(newUser.username, newUser.password);
  }

  /**
   * Get an admin token for administrative operations.
   * Uses the admin credentials against the master realm.
   */
  public async getAdminToken(): Promise<TokenResponse> {
    return fetch(`${this.url}/realms/master/protocol/openid-connect/token`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
      },
      body: new URLSearchParams({
        grant_type: 'password',
        client_id: 'admin-cli',
        username: this.admin.username,
        password: this.admin.password,
      }),
    }).then((data) => data.json());
  }

  /**
   * Get the user info.
   */
  public async getUserInfo(token: string): Promise<UserInfoResponse> {
    return fetch(
      `${this.url}/realms/${this.realm}/protocol/openid-connect/userinfo`,
      {
        method: 'GET',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
          Authorization: `Bearer ${token}`,
        },
      },
    ).then((data) => data.json());
  }

  /**
   * Get a user token from username and password credentials.
   * Uses the default realm for authentication.
   */
  public async getUserToken(
    username: string,
    password: string,
  ): Promise<TokenResponse> {
    return fetch(
      `${this.url}/realms/${this.realm}/protocol/openid-connect/token`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded',
        },
        body: new URLSearchParams({
          grant_type: 'password',
          client_id: 'francenuage',
          username: username,
          password: password,
          scope: 'openid',
        }),
      },
    ).then((data) => data.json());
  }
}
