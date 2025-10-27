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
   * Create a new user.
   */
  public async createUser(
    data?: Partial<User>,
    realm?: string,
  ): Promise<TokenResponse> {
    const token = (await this.getAdminToken()).access_token;
    const newUser = { ...user({ password: 'password' }), ...data };

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

    if (!response.ok) {
      throw new Error(`could not create user -- ${await response.text()}`);
    }

    if (!newUser.username) {
      throw new Error('missing username for authentication');
    }

    if (!newUser.password) {
      throw new Error('missing password for authentication');
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
