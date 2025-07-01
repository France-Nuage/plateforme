import { Datacenter } from '@/types';

/**
 * Define datacenter actions.
 */
export interface DatacenterService {
  /** List the available datacenters */
  list: () => Promise<Datacenter[]>;
}
