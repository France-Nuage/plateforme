import { UserManager } from "oidc-client-ts";

export const userManager = new UserManager({
  accessTokenExpiringNotificationTimeInSeconds: 60,
  authority: process.env.NEXT_PUBLIC_OIDC_PROVIDER_URL!,
  automaticSilentRenew: true,
  client_id: process.env.NEXT_PUBLIC_OIDC_CLIENT_ID!,
  client_secret: process.env.NEXT_PUBLIC_OIDC_CLIENT_SECRET!,
  redirect_uri: `${process.env.NEXT_PUBLIC_CONSOLE_URL}/auth/redirect/${process.env.NEXT_PUBLIC_OIDC_PROVIDER_NAME}`,
  silent_redirect_uri: `${process.env.NEXT_PUBLIC_CONSOLE_URL}/auth/silent-redirect/${process.env.NEXT_PUBLIC_OIDC_PROVIDER_NAME}`,
  response_type: 'code',
  scope: 'openid profile email',
});
