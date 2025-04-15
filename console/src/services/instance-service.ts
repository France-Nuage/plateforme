import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { InstancesClient } from "../protocol/instances.client";
import { CreateInstanceRequest, CreateInstanceResponse, InstanceInfo } from "../protocol/instances";

const transport = new GrpcWebFetchTransport({
  baseUrl: import.meta.env.VITE_CONTROLPLANE_URL,
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

export function create(options: CreateInstanceRequest): Promise<CreateInstanceResponse> {
  return new Promise((resolve, reject) => {
    client.createInstance(options).response.then(resolve).catch(reject)
  })
}
