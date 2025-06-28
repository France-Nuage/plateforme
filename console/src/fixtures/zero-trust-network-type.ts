import { faker } from '@faker-js/faker';

import { ZeroTrustNetworkType } from '@/types';

export const acmeZeroTrustNetworkType: ZeroTrustNetworkType = {
  id: '00000000-0000-0000-0000-000000000000',
  name: 'untrustworthy-coyote',
};

export const zeroTrustNetworkType = (): ZeroTrustNetworkType => ({
  id: faker.string.uuid(),
  name: `ZeroTrust ${faker.commerce.productName()}`,
});

export const zeroTrustNetworkTypes = (count: number): ZeroTrustNetworkType[] =>
  [...Array(count)].map(zeroTrustNetworkType);
