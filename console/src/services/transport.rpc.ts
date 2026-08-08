import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import config from '@/config';
import { createUnaryAuthInterceptor } from '@/middlewares';
import { AppStore } from '@/store';

import { refreshSession } from './bff-auth';

/**
 * Configures the gRPC transport with cookie-based authentication and bounded
 * session recovery.
 *
 * This function creates a transport instance that:
 * - Sends the httpOnly session cookie with every call (`credentials: 'include'`),
 *   so the API is authenticated without any token held in JavaScript
 * - Recovers from an expired session once, via `/auth/refresh` + a single replay,
 *   and otherwise clears the auth state (the page guard then redirects to login)
 * - Uses binary format for optimized payload size
 *
 * @param store - The Redux store instance for accessing/clearing authentication state
 * @returns Configured GrpcWebFetchTransport instance
 */
export function configureTransport(store: AppStore) {
  return new GrpcWebFetchTransport({
    /**
     * Use the controlplane as the base url.
     */
    baseUrl: config.controlplane,

    /**
     * Send the httpOnly session cookie cross-subdomain so gRPC calls are
     * authenticated by the cookie rather than a bearer token.
     */
    fetchInit: { credentials: 'include' },

    /**
     * Use the `binary` format instead of `text` (less readable but lighter).
     */
    format: 'binary',

    /**
     * Recover from an expired session (refresh + single replay), then fail
     * closed by clearing the auth state.
     */
    interceptors: [createUnaryAuthInterceptor(store, refreshSession)],
  });
}
