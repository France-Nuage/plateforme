import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Zone as RpcZone } from '../../generated/rpc/compute';
import { ZonesClient } from '../../generated/rpc/compute.client';
import { Zone, ZoneFormValue } from '../../models';
import { ZoneService } from '../api';

export class ZoneRpcService implements ZoneService {
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
  public async create(data: ZoneFormValue): Promise<Zone> {
    const { zone } = await this.client.create(data).response;
    return zone!;
  }

  /** @inheritdoc */
  public async list(): Promise<Zone[]> {
    const { zones } = await this.client.list({}).response;
    return zones.map(fromRpcZone);
  }
}

/**
 * Convert a protocol zone into a concrete Datacenter.
 */
function fromRpcZone({ id, name }: RpcZone): Zone {
  return { id, name };
}
