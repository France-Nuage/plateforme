import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Hypervisor as RpcHypervisor } from '../../generated/rpc/compute';
import { HypervisorsClient } from '../../generated/rpc/compute.client';
import { Hypervisor, HypervisorFormValue } from '../../models';
import { HypervisorService } from '../api';

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
  public async list(): Promise<Hypervisor[]> {
    const { hypervisors } = await this.client.list({}).response;
    return hypervisors.map(fromRpcHypervisor);
  }

  /**
   * @inheritdoc
   */
  public async register({
    authorizationToken = '',
    storageName,
    organizationId,
    url,
    zoneId,
  }: HypervisorFormValue): Promise<Hypervisor> {
    const { hypervisor } = await this.client.register({
      authorizationToken,
      organizationId,
      storageName,
      url,
      zoneId,
    }).response;
    return fromRpcHypervisor(hypervisor!);
  }
}

// Converts a protocol Hypervisor into a concrete Hypervisor.
function fromRpcHypervisor(hypervisor: RpcHypervisor): Hypervisor {
  return {
    id: hypervisor.id,
    organizationId: hypervisor.organizationId,
    storageName: hypervisor.storageName,
    url: hypervisor.url,
    zoneId: hypervisor.zoneId,
  };
}
