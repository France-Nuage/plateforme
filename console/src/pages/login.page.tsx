import { Button, Center, Spinner, Text, VStack } from '@chakra-ui/react';
import { FunctionComponent, useEffect, useState } from 'react';

import { userManager } from '@/services';

/**
 * Login page component.
 *
 * This page automatically redirects users to the OIDC provider (Keycloak)
 * for authentication. A loading spinner is displayed during the redirect.
 * If the redirect fails, an error message with a retry button is shown.
 */
export const LoginPage: FunctionComponent = () => {
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    userManager.signinRedirect().catch((err: Error) => {
      console.error('OIDC redirect failed:', err);
      setError(
        'Failed to redirect to authentication provider. Please try again.',
      );
    });
  }, []);

  if (error) {
    return (
      <Center h="100vh">
        <VStack gap={4}>
          <Text color="red.500">{error}</Text>
          <Button onClick={() => window.location.reload()}>Retry</Button>
        </VStack>
      </Center>
    );
  }

  return (
    <Center h="100vh">
      <Spinner size="xl" color="blue.solid" />
    </Center>
  );
};
