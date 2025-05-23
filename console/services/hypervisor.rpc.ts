import { HypervisorsClient } from "@/protocol/hypervisors.client";
import { Hypervisor as RpcHypervisor } from "@/protocol/hypervisors";
import { Hypervisor, HypervisorFormValue } from "@/types";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { HypervisorService } from "./hypervisor.interface";
import { transport } from "./transport.rpc";

export class HypervisorRpcService implements HypervisorService {
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

  public register({
    authorizationToken = "",
    storageName,
    url,
  }: HypervisorFormValue): Promise<Hypervisor> {
    return this.client
      .registerHypervisor({ authorizationToken, storageName, url })
      .response.then(({ hypervisor }) => fromRpcHypervisor(hypervisor!));
  }
}

// Exports an instance of the service.
export const hypervisorsRpcService = new HypervisorRpcService(transport);

// Converts a protocol Hypervisor into a concrete Hypervisor.
function fromRpcHypervisor(hypervisor: RpcHypervisor): Hypervisor {
  return {
    id: hypervisor.id,
    storageName: hypervisor.storageName,
    url: hypervisor.url,
  };
}
