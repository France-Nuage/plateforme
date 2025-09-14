import { FunctionComponent, ReactNode, useEffect, useState } from 'react';

import {
  clearAuthenticationState,
  parseOidcUser,
  setOIDCUser,
} from '@/features';
import { useAppDispatch } from '@/hooks';
import { registerUserEvents, userManager } from '@/services';
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

  useEffect(() => {
    registerUserEvents(dispatch);
  }, [dispatch]);

  // Attempt to retrieve the persisted user, then mark the app as loaded
  useEffect(() => {
    userManager
      .getUser()
      .then(async (user) => {
        // No user found: fresh state, user has never logged in or was previously cleared
        if (!user) {
          return dispatch(clearAuthenticationState());
        }
        // User exists and tokens are still valid: dispatch to store
        if (user && !user.expired) {
          return dispatch(setOIDCUser(parseOidcUser(user)));
        }
        // User exists but tokens are expired: attempt silent refresh (common after browser restart)
        if (user && user.expired) {
          return userManager
            .signinSilent()
            .then((user) => {
              if (user) {
                dispatch(setOIDCUser(parseOidcUser(user)));
              } else {
                // Silent refresh returned null - throw error to trigger centralized cleanup
                throw new Error('Silent refresh returned null user');
              }
            })
            .catch(() => {
              // Centralized cleanup: Remove the expired user from storage and clear auth state
              userManager.removeUser();
              dispatch(clearAuthenticationState());
            });
        }
        // todo: should never reach that point
        throw new Error(`Unexpected user state: ${JSON.stringify(user)}`);
      })
      .catch((error) => toaster.create({ title: error }))
      .finally(() => setUserRetrieved(true));
  }, [dispatch]);

  // Render the remaining tree only after user retrieval
  return isUserStateRetrieved ? children : null;
};
