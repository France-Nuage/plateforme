import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import {
  ResourcesClient,
  Organization as RpcOrganization,
} from '../../generated/rpc';
import { Organization, OrganizationFormValue } from '../../models';
import { OrganizationService } from '../api';

export class OrganizationRpcService implements OrganizationService {
  /**
   * The gRPC resources client
   */
  private client: ResourcesClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ResourcesClient(transport);
  }

  /** @inheritdoc */
  public create(data: OrganizationFormValue): Promise<Organization> {
    return this.client
      .createOrganization({
        name: data.name,
      })
      .response.then(({ organization }) => fromRpcOrganization(organization!));
  }
  /** @inheritdoc */
  public list(): Promise<Organization[]> {
    return this.client
      .listOrganizations({})
      .response.then(({ organizations }) =>
        organizations.map(fromRpcOrganization),
      );
  }
}

/**
 * Convert a protocol organization into a concrete Organization.
 */
function fromRpcOrganization(organization: RpcOrganization): Organization {
  return {
    id: organization.id,
    name: organization.name,
  };
}
