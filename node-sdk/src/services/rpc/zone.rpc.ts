import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Zone as RpcZone } from '../../generated/rpc/compute';
import { ZonesClient } from '../../generated/rpc/compute.client';
import { Zone } from '../../models';
import { ZoneService } from '../api';

export class DatacenterRpcService implements ZoneService {
  /**
   * The gRPC resources client
   */
  private client: ZonesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ZonesClient(transport);
  }

  /** @inheritdoc */
  public async list(): Promise<Zone[]> {
    const { zones } = await this.client.list({}).response;
    return zones.map(fromRpcDatacenter);
  }
}

/**
 * Convert a protocol datacenter into a concrete Datacenter.
 */
function fromRpcDatacenter({ id, name }: RpcZone): Zone {
  return { id, name };
}
