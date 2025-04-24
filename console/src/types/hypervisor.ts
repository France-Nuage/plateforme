export interface Hypervisor {
  // The hypervisor id.
  id: string;

  // The hypervisor authorization token.
  authorizationToken?: string;

  // The hypervisor default storage name.
  storageName: string;

  // The hypervisor url.
  url: string;
}
