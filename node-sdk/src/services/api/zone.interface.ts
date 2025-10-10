import { Zone } from '../../models';

/**
 * Define zone actions.
 */
export interface ZoneService {
  /** List the available zones */
  list: () => Promise<Zone[]>;
}
