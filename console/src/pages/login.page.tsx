import { Center, Spinner } from '@chakra-ui/react';
import { FunctionComponent, useEffect } from 'react';

import { loginRedirect } from '@/services';

/**
 * Login page component.
 *
 * Immediately redirects the browser to the control plane's `/auth/login`, which
 * runs the confidential-client authorization-code flow against the identity
 * provider. A loading spinner is displayed for the brief moment before the
 * navigation happens.
 */
export const LoginPage: FunctionComponent = () => {
  useEffect(() => {
    loginRedirect();
  }, []);

  return (
    <Center h="100vh">
      <Spinner size="xl" color="blue.solid" />
    </Center>
  );
};
