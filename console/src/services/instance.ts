import { InstancesClient } from "@/protocol/instances.client";
import { Instance } from "@/types";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

export class InstanceService {
  /**
   * The gRPC instances client.
   */
  private client: InstancesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new InstancesClient(transport);
  }

  public list(): Promise<Instance[]> {
    return this.client
      .listInstances({})
      .response.then(({ instances }) =>
        instances.map(({ id, name }) => ({ id, name })),
      );
  }
}
