import { RpcError, RpcOptions } from '@protobuf-ts/runtime-rpc';
import { debounce } from 'lodash';

import { ERROR_DEBOUNCE_WAIT } from '@/constants';
import { logout } from '@/features';
import { AppStore } from '@/store';
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
export function applyAuthenticationHeader(
  options: RpcOptions,
  store: AppStore,
) {
  return {
    ...options,
    meta: {
      ...options.meta,
      ...(store.state.authentication.token && {
        Authorization: `Bearer ${store.state.authentication.token}`,
      }),
    },
  };
}

/**
 * Handles RPC errors from gRPC service calls.
 *
 * Processes errors, maps unregistered user messages, notifies users
 * via toasts, and handles authentication failures by logging out users.
 *
 * @param error - The RPC error from gRPC service
 * @param store - Redux store for dispatching actions
 */
export function handleRpcError(error: RpcError, store: AppStore) {
  error.message = decodeURIComponent(error.message);
  error = mapUnregisteredErrorToAlphaVersionContextualError(error);
  notify(error.code, error.message);
  switch (error.code) {
    case 'UNAUTHENTICATED':
      store.dispatch(logout());
      break;
    default:
      console.error(
        `unhandled error code: "${error.code}"`,
        JSON.stringify(error),
      );
      throw error;
  }
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

/**
 * Maps unregistered user errors to e contextual error with support contact.
 *
 * Temporary workaround for alpha version - users must be manually created in database.
 * Will be removed once SpiceDB and proper user onboarding are implemented.
 *
 * @param error - RPC error to transform
 * @returns Modified error with contextual message or original error
 */
function mapUnregisteredErrorToAlphaVersionContextualError(error: RpcError) {
  const regex = /^user\s+(\S+@\S+\.\S+)\s+is\s+not\s+registered$/;
  const match = error.message.match(regex);
  if (match) {
    const email = match[1];
    error.code = 'ACCESS DENIED';
    error.message = `Email "${email}" is not registered. Contact support@france-nuage.fr for alpha access.`;
  }

  return error;
}
