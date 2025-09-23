import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  ZeroTrustNetworkType as RpcZeroTrustNetworkType,
  ZeroTrustNetworkTypesClient,
} from '../../generated/rpc';
import { ZeroTrustNetworkType } from '../../models';
import { ZeroTrustNetworkTypeService } from '../api';

export class ZeroTrustNetworkTypeRpcService
  implements ZeroTrustNetworkTypeService
{
  /**
   * The gRPC resources client
   */
  private client: ZeroTrustNetworkTypesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ZeroTrustNetworkTypesClient(transport);
  }

  /** @inheritdoc */
  public list(): Promise<ZeroTrustNetworkType[]> {
    return this.client
      .list({})
      .response.then(({ zeroTrustNetworkTypes }) =>
        zeroTrustNetworkTypes.map(fromRpcZeroTrustNetworkType),
      );
  }
}

/**
 * Convert a protocol organization into a concrete Organization.
 */
function fromRpcZeroTrustNetworkType(
  zeroTrustNetworkType: RpcZeroTrustNetworkType,
): ZeroTrustNetworkType {
  return {
    id: zeroTrustNetworkType.id,
    name: zeroTrustNetworkType.name,
  };
}
