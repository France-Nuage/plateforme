// @vitest-environment jsdom
import { ChakraProvider, defaultSystem } from '@chakra-ui/react';
import { configureStore } from '@reduxjs/toolkit';
import { cleanup, render, screen } from '@testing-library/react';
import { Provider as ReduxProvider } from 'react-redux';
import { MemoryRouter, Route, Routes } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import { authenticationSlice } from '@/features';

import { AdminGuard } from './admin-guard';

afterEach(() => {
  cleanup();
});

/**
 * Renders {@link AdminGuard} as a route element with a guarded index child,
 * driving it off a real authentication store preloaded with `isAdmin`.
 */
function renderAdminGuard(isAdmin: boolean): void {
  const store = configureStore({
    preloadedState: {
      authentication: { authenticated: isAdmin, isAdmin, sessionError: false },
    },
    reducer: { [authenticationSlice.name]: authenticationSlice.reducer },
  });

  render(
    <ChakraProvider value={defaultSystem}>
      <ReduxProvider store={store}>
        <MemoryRouter initialEntries={['/']}>
          <Routes>
            <Route element={<AdminGuard />}>
              <Route index element={<div>zone-admin</div>} />
            </Route>
          </Routes>
        </MemoryRouter>
      </ReduxProvider>
    </ChakraProvider>,
  );
}

describe('AdminGuard', () => {
  it('renders the guarded routes for an admin', () => {
    renderAdminGuard(true);
    expect(screen.getByText('zone-admin')).not.toBeNull();
    expect(screen.queryByText('Accès refusé')).toBeNull();
  });

  it('renders the "Accès refusé" alert for a non-admin', () => {
    renderAdminGuard(false);
    expect(screen.getByText('Accès refusé')).not.toBeNull();
    expect(screen.queryByText('zone-admin')).toBeNull();
  });
});
