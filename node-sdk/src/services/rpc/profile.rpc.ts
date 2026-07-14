import { GrpcWebFetchTransport } from '@protobuf-ts/grpcweb-transport';

import { ProfileClient } from '../../generated/rpc/iam.client';
import { CurrentUser } from '../../models';
import { ProfileService } from '../api';

export class ProfileRpcService implements ProfileService {
  /**
   * The gRPC profile client.
   */
  private client: ProfileClient;

  /**
   * The class constructor.
   */
  constructor(transport: GrpcWebFetchTransport) {
    this.client = new ProfileClient(transport);
  }

  /**
   * @inheritdoc
   */
  public async getCurrentUser(): Promise<CurrentUser> {
    const { id, email, isAdmin } = await this.client.getCurrentUser({})
      .response;
    return { id, email, isAdmin };
  }
}
