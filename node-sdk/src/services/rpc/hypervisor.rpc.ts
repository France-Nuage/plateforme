import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  HypervisorsClient,
  Hypervisor as RpcHypervisor,
} from '../../generated/rpc';
import {
  Hypervisor,
  HypervisorFormValue,
  HypervisorService,
} from '../../types';

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

  /**
   * @inheritdoc
   */
  public list(): Promise<Hypervisor[]> {
    return this.client
      .listHypervisors({})
      .response.then(({ hypervisors }) => hypervisors.map(fromRpcHypervisor));
  }

  /**
   * @inheritdoc
   */
  public register({
    authorizationToken = '',
    datacenterId,
    storageName,
    organizationId,
    url,
  }: HypervisorFormValue): Promise<Hypervisor> {
    return this.client
      .registerHypervisor({
        authorizationToken,
        datacenterId,
        organizationId,
        storageName,
        url,
      })
      .response.then(({ hypervisor }) => fromRpcHypervisor(hypervisor!));
  }
}

// Converts a protocol Hypervisor into a concrete Hypervisor.
function fromRpcHypervisor(hypervisor: RpcHypervisor): Hypervisor {
  return {
    datacenterId: hypervisor.datacenterId,
    id: hypervisor.id,
    organizationId: hypervisor.organizationId,
    storageName: hypervisor.storageName,
    url: hypervisor.url,
  };
}
