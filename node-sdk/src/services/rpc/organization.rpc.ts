import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Organization as RpcOrganization } from '../../generated/rpc/resourcemanager';
import { OrganizationsClient } from '../../generated/rpc/resourcemanager.client';
import { Organization, OrganizationFormValue } from '../../models';
import { OrganizationService } from '../api';

export class OrganizationRpcService implements OrganizationService {
  /**
   * The gRPC resources client
   */
  private client: OrganizationsClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new OrganizationsClient(transport);
  }

  /** @inheritdoc */
  public create(data: OrganizationFormValue): Promise<Organization> {
    return this.client
      .create(data)
      .response.then(({ organization }) => fromRpcOrganization(organization!));
  }

  /** @inheritdoc */
  public list(): Promise<Organization[]> {
    return this.client
      .list({})
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
