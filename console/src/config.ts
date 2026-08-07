/**
 * Application configuration, resolved at runtime from `window.__RUNTIME_CONFIG__`
 * (see `public/config.js`), so a single build is portable across environments.
 */
type RuntimeConfig = {
  controlplaneUrl: string;
  oidcClientId: string;
  oidcProviderName: string;
  oidcProviderUrl: string;
  applicationMode: string;
};

const runtime = (window as unknown as { __RUNTIME_CONFIG__?: RuntimeConfig })
  .__RUNTIME_CONFIG__;

if (!runtime) {
  throw new Error(
    'Runtime configuration is missing: /config.js did not define window.__RUNTIME_CONFIG__',
  );
}

export default {
  controlplane: runtime.controlplaneUrl,
  mode: runtime.applicationMode,
  oidc: {
    clientId: runtime.oidcClientId,
    name: runtime.oidcProviderName,
    url: runtime.oidcProviderUrl,
  },
};
