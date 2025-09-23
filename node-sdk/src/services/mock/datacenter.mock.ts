import { acmeDatacenter, datacenters } from '../../fixtures';
import { DatacenterService } from '../api';

/**
 * The mock implementation of the datacenter service.
 */
export class DatacenterMockService implements DatacenterService {
  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeDatacenter, ...datacenters(2)]);
  }
}

/**
 * The instance of the datacenter mock service.
 */
export const datacenterMockService = new DatacenterMockService();
