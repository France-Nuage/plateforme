import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

export function transport(url: string, token: string): GrpcWebFetchTransport {
  return new GrpcWebFetchTransport({
    baseUrl: url,
    format: 'binary',
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

          call.response.catch((error) => {
            console.error(
              `[${error.name}] ${error.serviceName}/${error.methodName}: ${error.code}`,
            );
            console.log('------------------------');
            console.log(call.response);
            console.log('------------------------');
            console.log(input, options, token);
            console.log('------------------------');
          });

          return call;
        },
      },
    ],
  });
}
