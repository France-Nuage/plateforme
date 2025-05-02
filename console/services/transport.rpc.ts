import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

export const transport = new GrpcWebFetchTransport({
  baseUrl: process.env.CONTROLPLANE_URL!,
  format: "binary",
});
