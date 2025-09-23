import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  DatacentersClient,
  Datacenter as RpcDatacenter,
} from '../../generated/rpc';
import { Datacenter } from '../../models';
import { DatacenterService } from '../api';

export class DatacenterRpcService implements DatacenterService {
  /**
   * The gRPC resources client
   */
  private client: DatacentersClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new DatacentersClient(transport);
  }

  /** @inheritdoc */
  public list(): Promise<Datacenter[]> {
    return this.client
      .list({})
      .response.then(({ datacenters }) => datacenters.map(fromRpcDatacenter));
  }
}

/**
 * Convert a protocol datacenter into a concrete Datacenter.
 */
function fromRpcDatacenter(datacenter: RpcDatacenter): Datacenter {
  return {
    id: datacenter.id,
    name: datacenter.name,
  };
}
