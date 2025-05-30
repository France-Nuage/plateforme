import { UserManager } from 'oidc-client-ts';

import config from '@/config';

export const userManager = new UserManager({
  accessTokenExpiringNotificationTimeInSeconds: 60,
  authority: config.oidc.url,
  automaticSilentRenew: true,
  client_id: config.oidc.clientId,
  redirect_uri: `${window.location.origin}/auth/redirect/${config.oidc.name}`,
  silent_redirect_uri: `${window.location.origin}/auth/silent-redirect/${config.oidc.name}`,
  response_type: 'code',
  scope: 'openid profile email',
});
