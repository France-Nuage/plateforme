// @vitest-environment jsdom
import { cleanup, render, screen } from '@testing-library/react';
import { FunctionComponent } from 'react';
import { MemoryRouter, Route, Routes, useSearchParams } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import { Routes as RoutePath } from '@/types';

import { HomePage } from './home.page';

afterEach(() => {
  cleanup();
});

/**
 * Stub login route that echoes the `auth_error` it received, so a test can prove
 * the landing forwarded the reason through the query string.
 */
const LoginStub: FunctionComponent = () => {
  const [searchParams] = useSearchParams();
  return <div>{`login-error:${searchParams.get('auth_error')}`}</div>;
};

/**
 * Mounts {@link HomePage} as the index route, with stub destinations for the two
 * redirect targets, starting navigation at `entry`.
 */
function renderHome(entry: string): void {
  render(
    <MemoryRouter initialEntries={[entry]}>
      <Routes>
        <Route path={RoutePath.Home} element={<HomePage />} />
        <Route path={RoutePath.Login} element={<LoginStub />} />
        <Route
          path={RoutePath.ManagedServices}
          element={<div>managed-services</div>}
        />
      </Routes>
    </MemoryRouter>,
  );
}

describe('HomePage index redirect', () => {
  it('forwards auth_error to /login instead of dropping it', () => {
    renderHome('/?auth_error=state');

    expect(screen.getByText('login-error:state')).not.toBeNull();
    expect(screen.queryByText('managed-services')).toBeNull();
  });

  it('redirects to the managed services catalog when there is no auth_error', () => {
    renderHome('/');

    expect(screen.getByText('managed-services')).not.toBeNull();
    expect(screen.queryByText(/login-error:/)).toBeNull();
  });
});
