import { Alert, Button, Center, Spinner, Stack } from '@chakra-ui/react';
import { FunctionComponent, ReactNode, useEffect, useState } from 'react';

import { fetchSession } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';

export type UserProviderProps = {
  children: ReactNode;
};

/**
 * The user provider.
 *
 * On application load it reads the current session from the control plane
 * (`GET /auth/me`, over the httpOnly session cookie) to establish the auth
 * state (`authenticated` + `isAdmin`) before rendering the app. Authentication
 * is entirely server-side (confidential-client BFF); the browser never handles
 * a token, so there is no persistence or silent-renew to manage here — the
 * cookie and its refresh live on the server.
 *
 * A `/auth/me` server error (5xx) is handled here rather than propagated to the
 * page guard: the app renders an error state with a retry instead of the router
 * tree. Bouncing to `/login` on a 5xx would loop through the identity provider
 * back to the same failing endpoint.
 */
export const UserProvider: FunctionComponent<UserProviderProps> = ({
  children,
}) => {
  const dispatch = useAppDispatch();
  const [isUserStateRetrieved, setUserRetrieved] = useState<boolean>(false);
  const sessionError = useAppSelector(
    (state) => state.authentication.sessionError,
  );

  // Resolve the session once, then render regardless of the outcome: an
  // unauthenticated result simply lets the page guard redirect to `/login`.
  useEffect(() => {
    dispatch(fetchSession()).finally(() => setUserRetrieved(true));
  }, [dispatch]);

  // Show a spinner until the session has been resolved, rather than a blank
  // screen.
  if (!isUserStateRetrieved) {
    return (
      <Center h="100vh">
        <Spinner size="xl" color="blue.solid" />
      </Center>
    );
  }

  // The control plane returned a 5xx: surface it instead of mounting the router
  // (which would bounce an unauthenticated visitor to `/login` and loop).
  if (sessionError) {
    return (
      <Center h="100vh">
        <Alert.Root status="error" maxW="480px">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>Service momentanément indisponible</Alert.Title>
            <Alert.Description>
              <Stack gap="3" align="flex-start">
                Impossible de contacter le service d'authentification. Veuillez
                réessayer dans quelques instants.
                <Button
                  variant="outline"
                  size="sm"
                  colorPalette="red"
                  onClick={() => dispatch(fetchSession())}
                >
                  Réessayer
                </Button>
              </Stack>
            </Alert.Description>
          </Alert.Content>
        </Alert.Root>
      </Center>
    );
  }

  // Render the remaining tree only after the session has been resolved
  return children;
};
