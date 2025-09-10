import { FunctionComponent, ReactNode, useEffect, useState } from 'react';

import { clearAuthenticationState, setOIDCUser } from '@/features';
import { useAppDispatch } from '@/hooks';
import { userManager } from '@/services';
import { toaster } from '@/toaster';

export type UserProviderProps = {
  children: ReactNode;
};

/**
 * The user provider.
 *
 * On application load, it calls the userManager to attempt to retrieve the
 * persisted user, and dispatch an action depending on the user retrieval.
 * The `oidc-client-ts` library provides several security features which is why
 * we rely on it for authentication persistance, rather than a custom-brewed
 * redux persistance middleware.
 */
export const UserProvider: FunctionComponent<UserProviderProps> = ({
  children,
}) => {
  const dispatch = useAppDispatch();
  const [isUserStateRetrieved, setUserRetrieved] = useState<boolean>(false);

  // Attempt to retrieve the persisted user, then mark the app as loaded
  useEffect(() => {
    userManager
      .getUser()
      .then((user) => {
        if (user) {
          dispatch(setOIDCUser({ ...user }));
        } else {
          dispatch(clearAuthenticationState());
        }
        setUserRetrieved(true);
      })
      .catch((error) => toaster.create({ title: error }));
  }, [dispatch]);

  // Render the remaining tree only after user retrieval
  return isUserStateRetrieved ? children : null;
};
