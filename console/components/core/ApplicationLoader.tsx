import { clearAuthenticationState, setOIDCUser } from "@/features";
import { useAppDispatch } from "@/hooks";
import { userManager } from "@/services";
import { FunctionComponent, ReactNode, useEffect, useState } from "react";
import { toast } from "react-toastify";

export type ApplicationLoaderProps = {
  children: ReactNode;
}

export const ApplicationLoader: FunctionComponent<ApplicationLoaderProps> = ({ children }) => {
  const dispatch = useAppDispatch();
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    userManager.getUser().then((user) => {
      if (user) {
        dispatch(setOIDCUser({ ...user }))
      } else {
        dispatch(clearAuthenticationState())
      }
      setLoading(false);
    }).catch(toast.error);
  }, []);

  return loading ? 'loading' : children;
}
