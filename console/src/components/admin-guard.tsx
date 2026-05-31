import { Alert, Center } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Outlet } from 'react-router';

import { useAppSelector } from '@/hooks';

/**
 * Guard component that restricts access to admin users.
 *
 * Reads the `isAdmin` flag from the authentication slice (derived from the
 * Keycloak 'admin' realm role in the id_token) and renders the nested routes
 * for admins. Non-admin users see an "Accès refusé" message.
 *
 * NOTE: This is a UX-only gate. The server-side `users.is_admin` flag is
 * authoritative and enforced by the backend on every request.
 */
export const AdminGuard: FunctionComponent = () => {
  const isAdmin = useAppSelector((state) => state.authentication.isAdmin);

  if (!isAdmin) {
    return (
      <Center h="100%">
        <Alert.Root status="error" maxW="480px">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>Accès refusé</Alert.Title>
            <Alert.Description>
              Cette section est réservée aux administrateurs de la plateforme.
            </Alert.Description>
          </Alert.Content>
        </Alert.Root>
      </Center>
    );
  }

  return <Outlet />;
};
