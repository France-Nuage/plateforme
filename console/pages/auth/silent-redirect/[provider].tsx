"use client";

import { useAppDispatch } from "@/hooks";
import { userManager } from "@/services";
import { FunctionComponent, useEffect } from "react";

const AuthRedirect: FunctionComponent = () => {
  const dispatch = useAppDispatch();

  useEffect(() => {
    userManager.signinSilentCallback()
  }, [dispatch, userManager]);
  return null;
}

export default AuthRedirect;
