/**
 * Represents a user.
 *
 * The User object is returned from the OIDC provider, matching
 */
export type User = {
  /**
   * The user email.
   */
  email: string;

  /**
   * The user first name.
   */
  firstName: string;

  /**
   * The user last name.
   */
  lastName: string;

  /**
   * The user password, can be set on edition.
   */
  password?: string;

  /**
   * The user profile picture.
   */
  picture?: string;

  /**
   * The user username.
   */
  username?: string;
};
