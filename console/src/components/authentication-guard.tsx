import { FunctionComponent, ReactNode, useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';

import config from '@/config';
import { useAppSelector } from '@/hooks';

export type AuthenticationGuardProps = {
  children: ReactNode;
};

const authenticatedRoutes = ['/', '/instance'];
const guestRoutes = ['/login', `/auth/redirect/${config.oidc.name}`];

export const AuthenticationGuard: FunctionComponent<
  AuthenticationGuardProps
> = ({ children }) => {
  const authenticated = useAppSelector((state) => !!state.authentication.user);
  const location = useLocation();
  const navigate = useNavigate();
  console.log('in auth guard', location.pathname);

  useEffect(() => {
    if (authenticated && guestRoutes.includes(location.pathname)) {
      navigate('/', { replace: true });
    } else if (
      !authenticated &&
      authenticatedRoutes.includes(location.pathname)
    ) {
      navigate('/login', { replace: true });
    }
  }, [authenticated, location.pathname, navigate]);

  return children;
};
