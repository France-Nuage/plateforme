import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ZeroTrustNetworkType as RpcZeroTrustNetworkType } from '../../generated/rpc/infrastructure';
import { ZeroTrustNetworkTypesClient } from '../../generated/rpc/infrastructure.client';
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
  public async list(): Promise<ZeroTrustNetworkType[]> {
    const { zeroTrustNetworkTypes } = await this.client.list({}).response;
    return zeroTrustNetworkTypes.map(fromRpcZeroTrustNetworkType);
  }
}

/**
 * Convert a protocol zero trust network type into a concrete type.
 */
function fromRpcZeroTrustNetworkType({
  id,
  name,
}: RpcZeroTrustNetworkType): ZeroTrustNetworkType {
  return { id, name };
}
