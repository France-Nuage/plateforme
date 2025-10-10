import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ZeroTrustNetwork as RpcZeroTrustNetwork } from '../../generated/rpc/infrastructure';
import { ZeroTrustNetworksClient } from '../../generated/rpc/infrastructure.client';
import { ZeroTrustNetwork } from '../../models';
import { ZeroTrustNetworkService } from '../api';

export class ZeroTrustNetworkRpcService implements ZeroTrustNetworkService {
  /**
   * The gRPC resources client
   */
  private client: ZeroTrustNetworksClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ZeroTrustNetworksClient(transport);
  }

  /** @inheritdoc */
  public async list(): Promise<ZeroTrustNetwork[]> {
    const { zeroTrustNetworks } = await this.client.list({}).response;
    return zeroTrustNetworks.map(fromRpcZeroTrustNetwork);
  }
}

/**
 * Convert a protocol zero trust network into a concrete model.
 */
function fromRpcZeroTrustNetwork({
  id,
  name,
  zeroTrustNetworkTypeId,
}: RpcZeroTrustNetwork): ZeroTrustNetwork {
  return { id, name, zeroTrustNetworkTypeId };
}
