import { FunctionComponent, ReactNode, useEffect, useState } from 'react';

import { fetchSession } from '@/features';
import { useAppDispatch } from '@/hooks';

export type UserProviderProps = {
  children: ReactNode;
};

/**
 * The user provider.
 *
 * On application load it reads the current session from the control plane
 * (`GET /auth/me`, over the httpOnly session cookie) to establish the auth
 * state (`authenticated` + `isAdmin`) before rendering the app. Authentication
 * is entirely server-side (confidential-client BFF); the browser never handles
 * a token, so there is no persistence or silent-renew to manage here — the
 * cookie and its refresh live on the server.
 */
export const UserProvider: FunctionComponent<UserProviderProps> = ({
  children,
}) => {
  const dispatch = useAppDispatch();
  const [isUserStateRetrieved, setUserRetrieved] = useState<boolean>(false);

  // Resolve the session once, then render regardless of the outcome: an
  // unauthenticated result simply lets the page guard redirect to `/login`.
  useEffect(() => {
    dispatch(fetchSession()).finally(() => setUserRetrieved(true));
  }, [dispatch]);

  // Render the remaining tree only after the session has been resolved
  return isUserStateRetrieved ? children : null;
};
