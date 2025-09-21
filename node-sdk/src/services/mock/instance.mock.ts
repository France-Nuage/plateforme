import {
  acmeHypervisor,
  acmeOrganization,
  acmeZeroTrustNetwork,
  instance,
  instances,
} from '../../fixtures';
import { InstanceFormValue } from '../../types';
import { InstanceService } from '../../types';

export class InstanceMockService implements InstanceService {
  /** @inheritdoc */
  create(data: InstanceFormValue) {
    return Promise.resolve({ ...instance(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve(
      instances(5, {
        hypervisorId: acmeHypervisor.id,
        projectId: acmeOrganization.id,
        zeroTrustNetworkId: acmeZeroTrustNetwork.id,
      }),
    );
  }

  remove() {
    return Promise.resolve();
  }

  start() {
    return Promise.resolve();
  }

  stop() {
    return Promise.resolve();
  }
}

export const instanceMockService = new InstanceMockService();
