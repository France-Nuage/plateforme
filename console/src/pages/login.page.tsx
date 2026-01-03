import { Center, Spinner } from '@chakra-ui/react';
import { FunctionComponent, useEffect } from 'react';

import { userManager } from '@/services';

/**
 * Login page component.
 *
 * This page automatically redirects users to the OIDC provider (Keycloak)
 * for authentication. A loading spinner is displayed during the redirect.
 */
export const LoginPage: FunctionComponent = () => {
  useEffect(() => {
    userManager.signinRedirect();
  }, []);

  return (
    <Center h="100vh">
      <Spinner size="xl" color="blue.solid" />
    </Center>
  );
};
