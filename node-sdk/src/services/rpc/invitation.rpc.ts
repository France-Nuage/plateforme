import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { Invitation as RpcInvitation } from '../../generated/rpc/iam';
import { InvitationsClient } from '../../generated/rpc/iam.client';
import { Invitation, InvitationFormValue } from '../../models';
import { InvitationService } from '../api';

export class InvitationRpcService implements InvitationService {
  /**
   * The gRPC instances client.
   */
  private client: InvitationsClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new InvitationsClient(transport);
  }

  /**
   * @inheritdoc
   */
  public async create({
    email,
    organizationId,
  }: InvitationFormValue): Promise<Invitation> {
    const { invitation } = await this.client.create({
      email,
      organizationId,
    }).response;
    return fromRpcInvitation(invitation!);
  }

  /**
   * @inheritdoc
   */
  public async list(): Promise<Invitation[]> {
    const { invitations } = await this.client.list({}).response;
    return invitations.map(fromRpcInvitation);
  }
}

// Converts a protocol Invitation into a concrete invitation.
function fromRpcInvitation(invitation: RpcInvitation): Invitation {
  return {
    id: invitation.id,
    organizationId: invitation.organizationId,
    userId: invitation.userId,
  };
}
