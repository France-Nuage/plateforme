"use client";

import { useAppDispatch } from "@/hooks";
import { userManager } from "@/services";
import { FunctionComponent, useEffect } from "react";

/**
 * The AuthRedirect page component.
 *
 * This page is the redirection target for the silent-redirect oidc
 * authentication flows, which allows keeping users signed in.
 */
const AuthRedirect: FunctionComponent = () => {
  const dispatch = useAppDispatch();

  useEffect(() => {
    userManager.signinSilentCallback();
  }, [dispatch, userManager]);
  return null;
};

export default AuthRedirect;
