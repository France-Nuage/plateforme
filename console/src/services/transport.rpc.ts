import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import config from '@/config';

export const transport = new GrpcWebFetchTransport({
  baseUrl: config.controlplane,
  format: "binary",
});
