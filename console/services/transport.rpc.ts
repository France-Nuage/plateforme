import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

if (!process.env.NEXT_PUBLIC_CONTROLPLANE_URL) {
  throw new Error("missing env var CONTROLPLANE_URL");
}

export const transport = new GrpcWebFetchTransport({
  baseUrl: process.env.NEXT_PUBLIC_CONTROLPLANE_URL,
  format: "binary",
});
