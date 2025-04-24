import { HypervisorsClient } from "@/protocol/hypervisors.client";
import { Hypervisor } from "@/types";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";

export class HypervisorService {
  /**
   * The gRPC instances client.
   */
  private client: HypervisorsClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new HypervisorsClient(transport);
  }

  public list(): Promise<Hypervisor[]> {
    return this.client.listHypervisors({}).response.then(({ hypervisors }) =>
      hypervisors.map(({ id, storageName, url }) => ({
        id,
        storageName,
        url,
      })),
    );
  }

  public create({
    authorizationToken = "",
    storageName,
    url,
  }: Omit<Hypervisor, "id">): Promise<void> {
    return this.client
      .registerHypervisor({ authorizationToken, storageName, url })
      .response.then(() => {});
  }
}
