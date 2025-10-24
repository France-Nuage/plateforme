/**
 * Represent an invitation.
 */
export type Invitation = {
  /**
   * The invitation id.
   */
  id: string;

  /**
   * The invitation organization id.
   */
  organizationId: string;

  /**
   * The user id.
   */
  userId: string;
};

/**
 * The invitation form creation/update value.
 */
export type InvitationFormValue = Pick<Invitation, 'organizationId'> & {
  email: string;
};
