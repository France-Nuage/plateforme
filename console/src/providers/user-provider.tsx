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
    pickAction()
      .then((action) => dispatch(action))
      .catch((error) => {
        console.error('[UserProvider] Failed to load user:', error);
        toaster.create({
          description: error?.message || String(error),
          title: 'Authentication Error',
          type: 'error',
        });
      })
      .finally(() => setUserRetrieved(true));
  }, [dispatch]);

  // Render the remaining tree only after user retrieval
  return isUserStateRetrieved ? children : null;
};

async function pickAction() {
  const user = await userManager.getUser();

  // No user found: fresh state, user has never logged in or was previously cleared
  if (!user) {
    return clearAuthenticationState();
  }

  // Check if user needs refresh (expired or missing profile data)
  const needsRefresh =
    user.expired ||
    !user.profile ||
    !user.profile.email ||
    !user.profile.given_name ||
    !user.profile.family_name;

  // User needs refresh (expired or missing profile): attempt silent signin
  if (needsRefresh) {
    const refreshedUser = await userManager.signinSilent();
    if (refreshedUser) {
      return setOIDCUser(parseOidcUser(refreshedUser));
    } else {
      // Cleanup: Remove the expired/incomplete user from storage and clear auth state
      userManager.removeUser();
      return clearAuthenticationState();
    }
  }

  // User exists and is complete: dispatch to store
  return setOIDCUser(parseOidcUser(user));
}
