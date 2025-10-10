import { acmeZone, zones } from '../../fixtures';
import { ZoneService } from '../api';

/**
 * The mock implementation of the datacenter service.
 */
export class DatacenterMockService implements ZoneService {
  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeZone, ...zones(2)]);
  }
}

/**
 * The instance of the datacenter mock service.
 */
export const datacenterMockService = new DatacenterMockService();
