import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ZeroTrustNetwork as RpcZeroTrustNetwork } from '@/generated/rpc/infrastructure';
import { ZeroTrustNetworksClient } from '@/generated/rpc/infrastructure.client';
import { ZeroTrustNetwork } from '@/types';

import { transport } from './transport.rpc';
import { ZeroTrustNetworkService } from './zero-trust-network.interface';

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
  public list(): Promise<ZeroTrustNetwork[]> {
    return this.client
      .list({})
      .response.then(({ zeroTrustNetworks }) =>
        zeroTrustNetworks.map(fromRpcZeroTrustNetwork),
      );
  }
}

export const zeroTrustNetworkRpcService = new ZeroTrustNetworkRpcService(
  transport,
);

/**
 * Convert a protocol organization into a concrete Organization.
 */
function fromRpcZeroTrustNetwork(
  zeroTrustNetwork: RpcZeroTrustNetwork,
): ZeroTrustNetwork {
  return {
    id: zeroTrustNetwork.id,
    name: zeroTrustNetwork.name,
    zeroTrustNetworkTypeId: zeroTrustNetwork.zeroTrustNetworkTypeId,
  };
}
