import { ZeroTrustNetworkType } from '@/types';

/**
 * Define zero trust network type actions.
 */
export interface ZeroTrustNetworkTypeService {
  /** List the available zero trust network types */
  list: () => Promise<ZeroTrustNetworkType[]>;
}
