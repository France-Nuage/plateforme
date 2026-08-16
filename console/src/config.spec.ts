import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

/**
 * `config.ts` reads `window.__RUNTIME_CONFIG__` at module-evaluation time, so
 * each case sets the runtime config, resets the module cache, and re-imports
 * `config` to trigger a fresh evaluation. This proves the boundary validation
 * fails loud on a missing or partial runtime config instead of degrading to a
 * silent `fetch('undefined/...')` state.
 */
describe('runtime configuration validation', () => {
  const globalWithWindow = globalThis as unknown as {
    window: { __RUNTIME_CONFIG__?: unknown };
  };
  let saved: unknown;

  beforeEach(() => {
    saved = globalWithWindow.window.__RUNTIME_CONFIG__;
    vi.resetModules();
  });

  afterEach(() => {
    globalWithWindow.window.__RUNTIME_CONFIG__ = saved;
    vi.resetModules();
  });

  it('resolves controlplane + mode from a complete runtime config', async () => {
    globalWithWindow.window.__RUNTIME_CONFIG__ = {
      applicationMode: 'rpc',
      controlplaneUrl: 'https://cp.example',
    };

    const mod = await import('./config');

    expect(mod.default.controlplane).toBe('https://cp.example');
    expect(mod.default.mode).toBe('rpc');
  });

  it('throws when __RUNTIME_CONFIG__ is missing entirely', async () => {
    globalWithWindow.window.__RUNTIME_CONFIG__ = undefined;

    await expect(import('./config')).rejects.toThrow(/did not define/);
  });

  it('throws when controlplaneUrl is missing (partial config fails loud)', async () => {
    globalWithWindow.window.__RUNTIME_CONFIG__ = { applicationMode: 'rpc' };

    await expect(import('./config')).rejects.toThrow(/controlplaneUrl/);
  });

  it('throws when applicationMode is missing (partial config fails loud)', async () => {
    globalWithWindow.window.__RUNTIME_CONFIG__ = {
      controlplaneUrl: 'https://cp.example',
    };

    await expect(import('./config')).rejects.toThrow(/applicationMode/);
  });
});
