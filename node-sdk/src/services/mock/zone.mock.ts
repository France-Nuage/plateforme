import { acmeZone, zones } from '../../fixtures';
import { ZoneService } from '../api';

/**
 * The mock implementation of the zone service.
 */
export class ZoneMockService implements ZoneService {
  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeZone, ...zones(2)]);
  }
}

/**
 * The instance of the zone mock service.
 */
export const zoneMockService = new ZoneMockService();
