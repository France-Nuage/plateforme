import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import config from '@/config';
import { applyAuthenticationHeader, handleRpcError } from '@/middlewares';

export const transport = new GrpcWebFetchTransport({
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
        const call = next(method, input, applyAuthenticationHeader(options));

        // apply the error handler
        call.response.catch(handleRpcError);

        // finally return the call
        return call;
      },
    },
  ],
});
