import { Zone, ZoneFormValue } from '../../models';

/**
 * Define zone actions.
 */
export interface ZoneService {
  /** Create a new zone */
  create: (data: ZoneFormValue) => Promise<Zone>;

  /** List the available zones */
  list: () => Promise<Zone[]>;
}
