/**
 * Application configuration, resolved at runtime from `window.__RUNTIME_CONFIG__`
 * (see `public/config.js`), so a single build is portable across environments.
 *
 * Authentication is handled entirely by the control plane (confidential-client
 * BFF): the browser reads its identity from `/auth/me` and the gRPC API is
 * authenticated by the httpOnly session cookie, never by a token held in
 * JavaScript. The console therefore holds no client-side OIDC configuration.
 */
type RuntimeConfig = {
  controlplaneUrl: string;
  applicationMode: string;
};

const runtime = (
  window as unknown as { __RUNTIME_CONFIG__?: Partial<RuntimeConfig> }
).__RUNTIME_CONFIG__;

if (!runtime) {
  throw new Error(
    'Runtime configuration is missing: /config.js did not define window.__RUNTIME_CONFIG__',
  );
}

// Validate the untrusted runtime object at this boundary and fail loud on a
// partial config, so a missing field surfaces as a named error at boot instead
// of degrading to `fetch('undefined/auth/me')` → a permanent, unexplained state.
const { applicationMode, controlplaneUrl } = runtime;
if (!controlplaneUrl) {
  throw new Error(
    'Runtime configuration is invalid: controlplaneUrl is missing or empty in /config.js',
  );
}
if (!applicationMode) {
  throw new Error(
    'Runtime configuration is invalid: applicationMode is missing or empty in /config.js',
  );
}

export default {
  controlplane: controlplaneUrl,
  mode: applicationMode,
};
