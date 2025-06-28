import { acmeZeroTrustNetworkType, zeroTrustNetworkTypes } from '@/fixtures';

import { ZeroTrustNetworkTypeService } from './zero-trust-network-type.interface';

/**
 * The mock implementation of the zero trust network type service.
 */
export class ZeroTrustNetworkTypeMockService
  implements ZeroTrustNetworkTypeService
{
  /** @inheritdoc */
  list() {
    return Promise.resolve([
      acmeZeroTrustNetworkType,
      ...zeroTrustNetworkTypes(2),
    ]);
  }
}

/**
 * The instance of the zero trust network type mock service.
 */
export const zeroTrustNetworkTypeMockService =
  new ZeroTrustNetworkTypeMockService();
