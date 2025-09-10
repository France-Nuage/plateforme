import { RpcError, RpcOptions } from '@protobuf-ts/runtime-rpc';
import { debounce } from 'lodash';

import {
  ACCESS_TOKEN_SESSION_STORAGE_KEY,
  ERROR_DEBOUNCE_WAIT,
} from '@/constants';
import { toaster } from '@/toaster';

/**
 * Applies authentication header to gRPC request options.
 *
 * This function serves as a middleware interceptor for gRPC calls, automatically
 * adding JWT Bearer token authentication to outgoing requests when a token is
 * available in session storage.
 *
 * @param options - The RPC options object to be enhanced with authentication
 * @returns Enhanced RPC options with Authorization header, or original options if no token
 *
 * ## Behavior
 *
 * - **Token Present**: Adds `Authorization: Bearer <token>` to request metadata
 * - **No Token**: Returns original options unchanged, allowing unauthenticated requests
 * - **Token Source**: Retrieves JWT token from browser session storage
 *
 * ## Usage
 *
 * This function is typically configured as an interceptor in the gRPC transport
 * configuration, ensuring all API calls automatically include authentication
 * when users are logged in.
 *
 * ```typescript
 * // Applied as transport interceptor
 * const transport = new GrpcWebFetchTransport({
 *   baseUrl: 'https://api.example.com',
 *   interceptors: [applyAuthenticationHeader]
 * });
 * ```
 */
export function applyAuthenticationHeader(options: RpcOptions) {
  // get the token from the store
  const token = sessionStorage.getItem(ACCESS_TOKEN_SESSION_STORAGE_KEY);

  // return the options as-is if token is not defined
  if (!token) {
    return options;
  }

  // return the options with the authorization meta
  return {
    ...options,
    meta: {
      ...options.meta,
      Authorization: `Bearer ${token}`,
    },
  };
}

/**
 * The handleRpcError middleware.
 *
 * This middleware is a function used as an interceptor in the
 * `services/transport.rpc.ts` grpc transport instance. It allows handling rpc
 * errors by:
 * - notifying the user through the UI using [toasts](https://www.chakra-ui.com/docs/components/toast)
 */
export function handleRpcError(error: RpcError) {
  notify(error.code, decodeURIComponent(error.message));
  throw error;
}

/**
 * Notify an error to the user.
 *
 * The notification is displayed as a toast in the application UI. It is also
 * debounced, meaning it is displayed only once every `ERROR_DEBOUNCE_WAIT`
 * milliseconds. This is particularly convenient when multiple unary calls
 * (a.k.a. rpcs) fail with the same error.
 *
 * The notify function is instantiated once in the module rather than on every
 * call to the `handleRpcError` to enable debouncing on the app level instead of
 * the call level.
 */
const notify = debounce((title: string, description: string) => {
  toaster.create({ description, title });
}, ERROR_DEBOUNCE_WAIT);
