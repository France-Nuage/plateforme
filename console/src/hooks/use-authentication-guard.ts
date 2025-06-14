import { useEffect } from 'react';
import { useLocation, useNavigate } from 'react-router';

import config from '@/config';
import { useAppSelector } from '@/hooks';

const authenticatedRoutes = ['/', '/instance'];
const guestRoutes = ['/login', `/auth/redirect/${config.oidc.name}`];

export const useAuthenticationGuard = () => {
  // Select state portions
  const authenticated = useAppSelector((state) => !!state.authentication.user);
  // Instantiate hooks
  const location = useLocation();
  const navigate = useNavigate();

  // Define an authentication guard effect
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
};
