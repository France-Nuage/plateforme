// @vitest-environment jsdom
import { ChakraProvider, defaultSystem } from '@chakra-ui/react';
import { configureStore } from '@reduxjs/toolkit';
import { cleanup, render, screen } from '@testing-library/react';
import { Provider as ReduxProvider } from 'react-redux';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { authenticationSlice } from '@/features';

import { UserProvider } from './user-provider';

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

/**
 * Mounts {@link UserProvider} over a real authentication store. On mount it
 * dispatches `fetchSession`, so `fetch` is stubbed to drive the outcome; the
 * assertions await the effect settling.
 */
function renderUserProvider(): void {
  const store = configureStore({
    reducer: { [authenticationSlice.name]: authenticationSlice.reducer },
  });

  render(
    <ChakraProvider value={defaultSystem}>
      <ReduxProvider store={store}>
        <UserProvider>
          <div>zone-app</div>
        </UserProvider>
      </ReduxProvider>
    </ChakraProvider>,
  );
}

describe('UserProvider', () => {
  it('renders the retry card and withholds the app when the control plane errors', async () => {
    // /auth/me answers 5xx → fetchSession flags sessionError. The provider MUST
    // render the retry card and NOT the children: mounting the app would let the
    // page guard bounce an unauthenticated visitor to /login and loop. Deleting
    // that guard would otherwise leave every other test green.
    vi.stubGlobal(
      'fetch',
      vi.fn(() =>
        Promise.resolve({ json: () => Promise.resolve({}), status: 500 }),
      ),
    );

    renderUserProvider();

    expect(
      await screen.findByText('Service momentanément indisponible'),
    ).not.toBeNull();
    expect(screen.queryByText('zone-app')).toBeNull();
  });

  it('renders the app once the session resolves without a server error', async () => {
    // A logged-out visitor (200 { authenticated: false }) is not an error: after
    // one bounded refresh attempt the provider renders the app (the page guard,
    // not the provider, then handles the redirect to /login).
    vi.stubGlobal(
      'fetch',
      vi.fn(() =>
        Promise.resolve({
          json: () => Promise.resolve({ authenticated: false }),
          status: 200,
        }),
      ),
    );

    renderUserProvider();

    expect(await screen.findByText('zone-app')).not.toBeNull();
    expect(screen.queryByText('Service momentanément indisponible')).toBeNull();
  });
});
