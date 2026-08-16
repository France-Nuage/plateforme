// @vitest-environment jsdom
import { ChakraProvider, defaultSystem } from '@chakra-ui/react';
import { configureStore } from '@reduxjs/toolkit';
import { cleanup, render, screen } from '@testing-library/react';
import { Provider as ReduxProvider } from 'react-redux';
import { MemoryRouter, Route, Routes } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import { ResourcesState, resourcesSlice } from '@/features';

import { OrganizationGuard } from './organization-guard';

afterEach(() => {
  cleanup();
});

/**
 * Renders {@link OrganizationGuard} as a route element with a guarded index
 * child, driving it off a real resources store preloaded into each branch.
 */
function renderOrganizationGuard(overrides: Partial<ResourcesState>): void {
  const store = configureStore({
    preloadedState: {
      resources: {
        organizations: [],
        organizationsError: false,
        organizationsLoaded: false,
        projects: [],
        ...overrides,
      },
    },
    reducer: { [resourcesSlice.name]: resourcesSlice.reducer },
  });

  render(
    <ChakraProvider value={defaultSystem}>
      <ReduxProvider store={store}>
        <MemoryRouter initialEntries={['/']}>
          <Routes>
            <Route element={<OrganizationGuard />}>
              <Route index element={<div>zone-app</div>} />
            </Route>
          </Routes>
        </MemoryRouter>
      </ReduxProvider>
    </ChakraProvider>,
  );
}

describe('OrganizationGuard', () => {
  it('renders the error state with a retry (not an infinite spinner) when the org fetch failed', () => {
    renderOrganizationGuard({
      organizationsError: true,
      organizationsLoaded: true,
    });

    expect(
      screen.getByText('Impossible de charger vos organisations'),
    ).not.toBeNull();
    expect(screen.getByText('Réessayer')).not.toBeNull();
    expect(screen.queryByText('zone-app')).toBeNull();
  });

  it('renders the app for a user who belongs to at least one organization', () => {
    renderOrganizationGuard({
      organizations: [{}] as unknown as ResourcesState['organizations'],
      organizationsLoaded: true,
    });

    expect(screen.getByText('zone-app')).not.toBeNull();
    expect(
      screen.queryByText('Impossible de charger vos organisations'),
    ).toBeNull();
  });

  it('withholds the app for a user without any organization', () => {
    renderOrganizationGuard({ organizations: [], organizationsLoaded: true });

    expect(screen.queryByText('zone-app')).toBeNull();
  });
});
