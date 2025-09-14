import { FunctionComponent, useEffect } from 'react';
import { useNavigate } from 'react-router';

import { parseOidcUser, setOIDCUser } from '@/features';
import { useAppDispatch } from '@/hooks';
import { userManager } from '@/services';

/**
 * The OidcRedirect page component.
 *
 * This page is the redirection target of the OIDC signin flow. The
 * `oidc-client-ts` internally handles all the logic of collecting query
 * parameters and finalizing the authentication code flow with PKCE.
 *
 * On completion, dispatch an action to populate the state with the
 * authenticated user and rewrite the browser history to set the user on the
 * home page.
 *
 * The redirection is a one-time process and consumes oidc state, so a user
 * should never visit this page more than one time per flow, thus the router
 * history rewrite, instead of a push.
 */
export const OidcRedirectPage: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  useEffect(() => {
    userManager.signinCallback().then((user) => {
      if (!user) {
        throw new Error('Error: user could not be retrieved.');
      }
      dispatch(setOIDCUser(parseOidcUser(user)));
      navigate('/', { replace: true });
    });
  }, [dispatch, navigate]);

  return <div>loading</div>;
};
