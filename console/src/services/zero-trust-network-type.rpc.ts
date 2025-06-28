import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ZeroTrustNetworkType as RpcZeroTrustNetworkType } from '@/generated/rpc/infrastructure';
import { ZeroTrustNetworkTypesClient } from '@/generated/rpc/infrastructure.client';
import { ZeroTrustNetworkType } from '@/types';

import { transport } from './transport.rpc';
import { ZeroTrustNetworkTypeService } from './zero-trust-network-type.interface';

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

export const zeroTrustNetworkTypeRpcService =
  new ZeroTrustNetworkTypeRpcService(transport);

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
