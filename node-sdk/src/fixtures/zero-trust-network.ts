import { faker } from '@faker-js/faker';

import { ZeroTrustNetwork } from '../types';

export const acmeZeroTrustNetwork: ZeroTrustNetwork = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'ACME ZTN',
  zeroTrustNetworkTypeId: '00000000-0000-0000-0000-000000000000',
};

export const zeroTrustNetwork = (): ZeroTrustNetwork => ({
  id: faker.string.uuid(),
  name: `ZeroTrust ${faker.commerce.productName()}`,
  zeroTrustNetworkTypeId: faker.string.uuid(),
});

export const zeroTrustNetworks = (count: number): ZeroTrustNetwork[] =>
  [...Array(count)].map(zeroTrustNetwork);
