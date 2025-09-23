import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  ZeroTrustNetwork as RpcZeroTrustNetwork,
  ZeroTrustNetworksClient,
} from '../../generated/rpc';
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
  public list(): Promise<ZeroTrustNetwork[]> {
    return this.client
      .list({})
      .response.then(({ zeroTrustNetworks }) =>
        zeroTrustNetworks.map(fromRpcZeroTrustNetwork),
      );
  }
}

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
