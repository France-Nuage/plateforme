/**
 * Represents a hypervisor.
 */
export type Hypervisor = {
  // The hypervisor id.
  id: string;

  // The slug of the organization the hypervisor belongs to.
  organizationSlug: string;

  // The hypervisor default storage name.
  storageName: string;

  // The hypervisor url.
  url: string;

  // The id of the zone the hypervisor belongs to.
  zoneId: string;
};

/**
 * The hypervisor form creation/update value.
 */
export type HypervisorFormValue = Pick<
  Hypervisor,
  'storageName' | 'organizationSlug' | 'url' | 'zoneId'
> & {
  authorizationToken: string;
};
