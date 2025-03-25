import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { HypervisorClient } from "../protocol/controlplane.client";
import { InstanceInfo } from "../protocol/controlplane";

const transport = new GrpcWebFetchTransport({
  baseUrl: "http://localhost",
  format: "binary",
});

const client = new HypervisorClient(transport);

export function list(): Promise<InstanceInfo[]> {
  return new Promise((resolve, reject) => {
    client
      .listInstances({})
      .response.then(({ result }) => {
        if (result.oneofKind === "success") {
          resolve(result.success.instances);
        } else if (result.oneofKind === "problem") {
          reject(result.problem);
        } else {
          throw new Error(`unexpected oneofKind: ${result.oneofKind}`);
        }
      })
      .catch((error) => {
        throw error;
      });
  });
}
