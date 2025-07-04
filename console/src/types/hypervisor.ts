/**
 * Represents a hypervisor.
 */
export type Hypervisor = {
  // The hypervisor id.
  id: string;

  // The hypervisor default storage name.
  storageName: string;

  // The id of the datacenter the hypervisor belongs to.
  datacenterId: string;

  // The id of the organization the hypervisor belongs to.
  organizationId: string;

  // The hypervisor url.
  url: string;
};

/**
 * The hypervisor form creation/update value.
 */
export type HypervisorFormValue = Pick<
  Hypervisor,
  'storageName' | 'datacenterId' | 'organizationId' | 'url'
> & {
  authorizationToken: string;
};
