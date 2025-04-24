import { HypervisorsClient } from "@/protocol/hypervisors.client";
import { Hypervisor as RpcHypervisor } from "@/protocol/hypervisors";
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
    return this.client
      .listHypervisors({})
      .response.then(({ hypervisors }) => hypervisors.map(fromRpcHypervisor));
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

// Converts a protocol Hypervisor into a concrete Hypervisor.
function fromRpcHypervisor(hypervisor: RpcHypervisor): Hypervisor {
  return {
    id: hypervisor.id,
    storageName: hypervisor.storageName,
    url: hypervisor.url,
  };
}
