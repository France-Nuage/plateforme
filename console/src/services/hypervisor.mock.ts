import { acmeHypervisor, hypervisor } from '@/fixtures';

import { HypervisorService } from './hypervisor.interface';

export class HypervisorMockService implements HypervisorService {
  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeHypervisor, hypervisor()]);
  }

  /** @inheritdoc */
  register() {
    return Promise.resolve(hypervisor());
  }
}

export const hypervisorMockService = new HypervisorMockService();
