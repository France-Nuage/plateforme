import { currentUser } from '../../fixtures/current-user';
import { ProfileService } from '../api';

/**
 * The mock implementation of the profile service.
 */
export class ProfileMockService implements ProfileService {
  /** @inheritdoc */
  getCurrentUser() {
    return Promise.resolve(currentUser());
  }
}

/**
 * The instance of the profile mock service.
 */
export const profileMockService = new ProfileMockService();
