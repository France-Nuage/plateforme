import { useAppSelector } from "@/hooks";
import { Router } from "next/router";
import { FunctionComponent, ReactNode } from "react";

export type AuthenticationGuardProps = {
  children: ReactNode;
  router: Router;
};

const authenticatedRoutes = ["/", "/instance"];
const guestRoutes = [
  "/login",
  `/auth/redirect/${process.env.NEXT_PUBLIC_OIDC_PROVIDER_NAME}`,
];

export const AuthenticationGuard: FunctionComponent<
  AuthenticationGuardProps
> = ({ children, router }) => {
  const authenticated = useAppSelector((state) => !!state.authentication.user);

  if (authenticated && guestRoutes.includes(router.pathname)) {
    router.replace("/");
  } else if (!authenticated && authenticatedRoutes.includes(router.pathname)) {
    router.replace("/login");
  }

  return children;
};
