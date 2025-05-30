/**
 * Represents a hypervisor.
 */
export type Hypervisor = {
  // The hypervisor id.
  id: string;

  // The hypervisor default storage name.
  storageName: string;

  // The hypervisor url.
  url: string;
};

/**
 * The hypervisor form creation/update value.
 */
export type HypervisorFormValue = Pick<Hypervisor, "storageName" | "url"> & {
  authorizationToken: string;
};
