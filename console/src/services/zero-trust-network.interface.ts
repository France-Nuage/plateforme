import { ZeroTrustNetwork } from '@/types';

/**
 * Define zero trust network type actions.
 */
export interface ZeroTrustNetworkService {
  /** List the available zero trust network */
  list: () => Promise<ZeroTrustNetwork[]>;
}
