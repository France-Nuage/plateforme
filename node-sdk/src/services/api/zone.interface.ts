import { Zone } from '../../models';

/**
 * Define datacenter actions.
 */
export interface ZoneService {
  /** List the available datacenters */
  list: () => Promise<Zone[]>;
}
