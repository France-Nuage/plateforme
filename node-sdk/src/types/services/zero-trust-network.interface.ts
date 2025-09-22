import { ZeroTrustNetwork } from '../models';

/**
 * Define zero trust network type actions.
 */
export interface ZeroTrustNetworkService {
  /** List the available zero trust network */
  list: () => Promise<ZeroTrustNetwork[]>;
}
