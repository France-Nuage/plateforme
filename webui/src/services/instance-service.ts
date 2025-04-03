import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { InstancesClient } from "../protocol/instances.client";
import { InstanceInfo } from "../protocol/instances";

const transport = new GrpcWebFetchTransport({
  baseUrl: "http://localhost",
  format: "binary",
});

const client = new InstancesClient(transport);

export function list(): Promise<InstanceInfo[]> {
  return new Promise((resolve, reject) => {
    client
      .listInstances({})
      .response.then(({ instances }) => resolve(instances))
      .catch((error) => reject(error));
  });
}
