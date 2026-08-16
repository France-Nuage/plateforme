import { FunctionComponent } from 'react';
import { Navigate, useSearchParams } from 'react-router';

import { Routes } from '@/types';

/**
 * Index route.
 *
 * The control plane sends every failed `/auth/callback` back to the console
 * origin as `/?auth_error=<reason>`, so the landing sees it first. When that
 * param is present it is forwarded to `/login` (kept in the URL, not stashed in
 * React state) so the login page can explain the failure instead of silently
 * bouncing back through the identity provider and looping. Otherwise the visitor
 * is redirected to the managed services catalog. Both redirects are declarative
 * so they resolve before URL-syncing effects run on `/`.
 */
export const HomePage: FunctionComponent = () => {
  const [searchParams] = useSearchParams();
  const authError = searchParams.get('auth_error');

  if (authError !== null) {
    return (
      <Navigate
        to={`${Routes.Login}?auth_error=${encodeURIComponent(authError)}`}
        replace
      />
    );
  }

  return <Navigate to={Routes.ManagedServices} replace />;
};
