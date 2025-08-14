import { FunctionComponent, useEffect, useState } from 'react';
import { Outlet, useNavigate } from 'react-router';

import { useAppSelector } from '@/hooks';
import { Routes } from '@/types';

export type PageGuardProps = {
  authenticated?: boolean;
};

/**
 * The PageGuard component.
 *
 * This component acts as a middleware positionned on every page and calls
 * general application hooks. These hooks need access to the router and the
 * state; this is why the logic has been positionned down in the React tree as
 * a higher-order component, rather than an actual middleware upper in the tree.
 */
export const PageGuard: FunctionComponent<PageGuardProps> = ({
  authenticated,
}) => {
  const isUserAuthenticated = useAppSelector(
    (state) => !!state.authentication.user,
  );
  const navigate = useNavigate();

  const [isLoading, setLoading] = useState(true);

  useEffect(() => {
    if (authenticated && !isUserAuthenticated) {
      navigate(Routes.Login);
    } else if (!authenticated && isUserAuthenticated) {
      navigate(Routes.Home);
    } else {
      setLoading(false);
    }
  }, [authenticated, isUserAuthenticated, navigate]);

  return isLoading ? <p></p> : <Outlet />;
};
