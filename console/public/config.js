// Runtime configuration (dev defaults). Overwritten by the container entrypoint.
window.__RUNTIME_CONFIG__ = {
  controlplaneUrl: 'https://controlplane.test',
  oidcClientId: 'francenuage',
  oidcProviderName: 'keycloak',
  oidcProviderUrl: 'https://keycloak.test/realms/francenuage',
  applicationMode: 'rpc',
};
