"use client";

import { setOIDCUser } from "@/features";
import { useAppDispatch } from "@/hooks";
import { userManager } from "@/services";
import { useRouter } from "next/router";
import { FunctionComponent, useEffect } from "react";
import { toast } from "react-toastify";

/**
 * The AuthRedirect page component.
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
const AuthRedirect: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const router = useRouter();

  useEffect(() => {
    userManager.signinCallback().then((user) => {
      if (!user) {
        throw new Error('Error: user could not be retrieved.');
      }
      dispatch(setOIDCUser(user));
      router.replace('/');
    }).catch((error: Error) => {
      toast.error(error.toString());
    });
  }, [dispatch, userManager]);

  return <div>loading</div>
};

export default AuthRedirect;
