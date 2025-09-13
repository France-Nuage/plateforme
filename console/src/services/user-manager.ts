import { UserManager } from 'oidc-client-ts';

import config from '@/config';
import { AppDispatch } from '@/store';

export const userManager = new UserManager({
  accessTokenExpiringNotificationTimeInSeconds: 60,
  authority: config.oidc.url,
  automaticSilentRenew: true,
  client_id: config.oidc.clientId,
  redirect_uri: `${window.location.origin}/auth/redirect/${config.oidc.name}`,
  response_type: 'code',
  scope: 'openid profile email',
  silent_redirect_uri: `${window.location.origin}/auth/silent-redirect/${config.oidc.name}`,
});

export function registerUserEvents(dispatch: AppDispatch) {
  userManager.events.addUserLoaded((user) => {
    console.log('user has been renewed!', user, dispatch);
  });
}
