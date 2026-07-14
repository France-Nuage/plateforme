import { CurrentUser } from '../../models';

/**
 * Exposes the authenticated caller's own identity, resolved server-side.
 */
export interface ProfileService {
  /**
   * Returns the current user as resolved by the control plane, including the
   * authoritative `isAdmin` flag.
   */
  getCurrentUser: () => Promise<CurrentUser>;
}
