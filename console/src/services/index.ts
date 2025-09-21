import { ServiceMode, Services, configureResolver } from '@france-nuage/sdk';

import { AppStore } from '@/store';

import { configureTransport } from './transport.rpc';

export * from './user-manager';

/**
 * Configures the complete service layer with store-aware transport.
 *
 * This function sets up the service architecture by:
 * 1. Creating a transport with store-based authentication
 * 2. Configuring service resolver with the transport
 * 3. Returning the complete service mapping
 *
 * Services are configured per-store instance to ensure proper authentication
 * state access and error handling integration.
 *
 * @param store - The Redux store instance
 * @returns Complete service resolver for all service modes
 */
export function configureServices(
  store: AppStore,
): Record<ServiceMode, Services> {
  const transport = configureTransport(store);
  const resolver = configureResolver(transport);

  return resolver;
}
