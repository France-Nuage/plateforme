import { configureResolver, ServiceMode } from '@france-nuage/sdk';
import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

const transport = new GrpcWebFetchTransport({
  baseUrl: process.env.CONTROLPLANE_URL!,
  interceptors: [
    {
      interceptUnary(next, method, input, options) {
        const call = next(
          method,
          input,
          options,
        );

        return call;
      }
    }
  ]
});

// @ts-ignore
const resolver = configureResolver(transport);

export const controlplane = resolver[ServiceMode.Rpc];


