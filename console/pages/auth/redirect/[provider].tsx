"use client";

import { setOIDCUser } from "@/features";
import { useAppDispatch } from "@/hooks";
import { userManager } from "@/services";
import { useRouter } from "next/router";
import { FunctionComponent, useEffect } from "react";
import { toast } from "react-toastify";

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
