import { Datacenter } from '../models';

/**
 * Define datacenter actions.
 */
export interface DatacenterService {
  /** List the available datacenters */
  list: () => Promise<Datacenter[]>;
}
