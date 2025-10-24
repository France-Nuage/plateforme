import { invitation, invitations } from '../../fixtures/invitation';
import { InvitationFormValue } from '../../models';
import { InvitationService } from '../api';

/**
 * The mock implementation of the invitation service.
 */
export class InvitationMockService implements InvitationService {
  /** @inheritdoc */
  create(data: InvitationFormValue) {
    return Promise.resolve({ ...invitation(), ...data });
  }

  /** @inheritdoc */
  list() {
    return Promise.resolve([...invitations(4)]);
  }
}

/**
 * The instance of the invitation mock service.
 */
export const invitationMockService = new InvitationMockService();
