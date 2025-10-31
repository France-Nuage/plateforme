import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

export function transport(url: string, token: string): GrpcWebFetchTransport {
  return new GrpcWebFetchTransport({
    baseUrl: url,
    interceptors: [
      {
        interceptUnary(next, method, input, options) {
          const call = next(method, input, {
            ...options,
            meta: {
              ...options.meta,
              Authorization: `Bearer ${token}`,
            },
          });

          call.response.catch((error) =>
            console.error(`[RPC] ${method}: ${error}`),
          );

          return call;
        },
      },
    ],
  });
}
