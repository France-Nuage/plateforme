import {
  acmeHypervisor,
  acmeOrganization,
  acmeZeroTrustNetwork,
  instance,
  instances,
} from '../../fixtures';
import { InstanceFormValue } from '../../models';
import { InstanceService } from '../api';

export class InstanceMockService implements InstanceService {
  /** @inheritdoc */
  clone() {
    return Promise.resolve(instance());
  }

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

  /** @inheritdoc */
  remove() {
    return Promise.resolve();
  }

  /** @inheritdoc */
  start() {
    return Promise.resolve();
  }

  /** @inheritdoc */
  stop() {
    return Promise.resolve();
  }

  /** @inheritdoc */
  update(_id: string, data: InstanceFormValue) {
    return Promise.resolve({ ...instance(), ...data });
  }
}

export const instanceMockService = new InstanceMockService();
