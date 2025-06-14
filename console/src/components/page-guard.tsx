import { FunctionComponent, ReactNode } from 'react';

import { useActiveParamsReconciliation, useAuthenticationGuard } from '@/hooks';

export type PageGuardProps = {
  children: ReactNode;
};

/**
 * The PageGuard component.
 *
 * This component acts as a middleware positionned on every page and calls
 * general application hooks. These hooks need access to the router and the
 * state; this is why the logic has been positionned down in the React tree as
 * a higher-order component, rather than an actual middleware upper in the tree.
 */
export const PageGuard: FunctionComponent<PageGuardProps> = ({ children }) => {
  // Call general application hooks
  useActiveParamsReconciliation();
  useAuthenticationGuard();

  return children;
};
