import { FunctionComponent } from 'react';
import { Navigate } from 'react-router';

import { Routes } from '@/types';

/**
 * Index route: redirects to the managed services catalog. The redirection is
 * declarative so it resolves before URL-syncing effects run on `/`.
 */
export const HomePage: FunctionComponent = () => (
  <Navigate to={Routes.ManagedServices} replace />
);
