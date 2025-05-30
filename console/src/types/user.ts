/**
 * Represents a user.
 *
 * The User object is returned from the OIDC provider, matching 
 */
export type User = {
  /**
   * The user email. 
   *
   * Requires the "email" scope.
   */
  email: string;

  /**
   * The user name.
   *
   * Requires the "profile" scope.
   */
  name?: string;

  /**
   * The user profile picture.
   *
   * Requires the "profile" scope.
   */
  picture?: string;
};
