import { Invitation, InvitationFormValue } from '../../models';

export interface InvitationService {
  list: () => Promise<Invitation[]>;
  create: (request: InvitationFormValue) => Promise<Invitation>;
}
