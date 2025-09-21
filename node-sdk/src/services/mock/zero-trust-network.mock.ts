import { acmeZeroTrustNetwork, zeroTrustNetworks } from '../../fixtures';
import { ZeroTrustNetworkService } from '../../types';

/**
 * The mock implementation of the zero trust network type service.
 */
export class ZeroTrustNetworkMockService implements ZeroTrustNetworkService {
  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeZeroTrustNetwork, ...zeroTrustNetworks(2)]);
  }
}

/**
 * The instance of the zero trust network type mock service.
 */
export const zeroTrustNetworkMockService = new ZeroTrustNetworkMockService();
