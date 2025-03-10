export interface AuthenticationToken {
  /**
   * The abilities provided by the token.
   */
  abilities: string[];

  /**
   * The token expiration time as a ISO8601 encoded string.
   */
  expiresAt: string;

  /**
   * The token expiration time as a ISO8601 encoded string.
   */
  lastUsedAd?: string;

  /**
   * The token name.
   */
  name?: string;

  /**
   * The token actual value.
   */
  token: string;

  /**
   * The token type.
   */
  type: "bearer";
}
