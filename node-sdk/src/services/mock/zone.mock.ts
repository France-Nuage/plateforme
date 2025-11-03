import { acmeZone, zone, zones } from '../../fixtures';
import { Zone, ZoneFormValue } from '../../models';
import { ZoneService } from '../api';

/**
 * The mock implementation of the zone service.
 */
export class ZoneMockService implements ZoneService {
  /** @inheritdoc */
  create(data: ZoneFormValue): Promise<Zone> {
    return Promise.resolve({ ...zone(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve([acmeZone, ...zones(2)]);
  }
}

/**
 * The instance of the zone mock service.
 */
export const zoneMockService = new ZoneMockService();
