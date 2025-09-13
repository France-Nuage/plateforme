import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import config from '@/config';
import { applyAuthenticationHeader, handleRpcError } from '@/middlewares';
import { AppStore } from '@/store';

/**
 * Configures the gRPC transport with store-aware authentication and error handling.
 *
 * This function creates a transport instance that:
 * - Automatically applies authentication headers from the Redux store
 * - Handles RPC errors with automatic logout on UNAUTHENTICATED responses
 * - Uses binary format for optimized payload size
 *
 * @param store - The Redux store instance for accessing authentication state
 * @returns Configured GrpcWebFetchTransport instance
 */
export function configureTransport(store: AppStore) {
  return new GrpcWebFetchTransport({
    /**
     * Use the controlplane as the base url.
     */
    baseUrl: config.controlplane,

    /**
     * Use the `binary` format instead of `text` (less readable but lighter).
     */
    format: 'binary',

    /**
     * Apply the `handleRpcError` middleware as an interceptor.
     */
    interceptors: [
      {
        interceptUnary(next, method, input, options) {
          // perform the unary operation
          const call = next(
            method,
            input,
            applyAuthenticationHeader(options, store),
          );

          // apply the error handler
          call.response.catch((error) => handleRpcError(error, store));

          // finally return the call
          return call;
        },
      },
    ],
  });
}
