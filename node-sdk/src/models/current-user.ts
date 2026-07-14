/**
 * Represents the authenticated caller as resolved by the control plane.
 *
 * Unlike {@link User} (which mirrors the OIDC profile), this is the
 * authoritative server-side identity: `isAdmin` reflects `users.is_admin` in
 * the control plane database, not an OIDC token claim.
 */
export type CurrentUser = {
  /**
   * The control plane user id.
   */
  id: string;

  /**
   * The user email.
   */
  email: string;

  /**
   * Whether the user holds platform-administration privileges.
   */
  isAdmin: boolean;
};
