/**
 * Test bootstrap.
 *
 * `src/config.ts` reads `window.__RUNTIME_CONFIG__` at import time (injected in
 * the browser by `public/config.js`). Under Vitest there is no such file, and
 * the `node` environment has no `window` at all, so importing anything that
 * transitively pulls in `config` would throw. Provide the runtime config here,
 * augmenting an existing `window` (jsdom) or creating one (node).
 */
const runtimeConfig = {
  applicationMode: 'live',
  controlplaneUrl: 'http://localhost',
};

const globalWithWindow = globalThis as {
  window?: { __RUNTIME_CONFIG__?: typeof runtimeConfig };
};

if (globalWithWindow.window) {
  globalWithWindow.window.__RUNTIME_CONFIG__ = runtimeConfig;
} else {
  globalWithWindow.window = { __RUNTIME_CONFIG__: runtimeConfig };
}
