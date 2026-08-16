import { Alert, Button, Center, Spinner, Stack } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Outlet } from 'react-router';

import { fetchAllOrganizations } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { PrivateBetaPage } from '@/pages';

/**
 * Guard component that restricts access to users with organization membership.
 *
 * While organizations are being fetched a loading spinner is shown. If the
 * fetch fails, an error state with a retry button is displayed instead of
 * spinning forever. Once loaded, users without any organization see the private
 * beta page; users who belong to at least one organization see the standard
 * application content.
 */
export const OrganizationGuard: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const organizationsLoaded = useAppSelector(
    (state) => state.resources.organizationsLoaded,
  );
  const organizationsError = useAppSelector(
    (state) => state.resources.organizationsError,
  );

  if (organizationsError) {
    return (
      <Center h="100vh">
        <Alert.Root status="error" maxW="480px">
          <Alert.Indicator />
          <Alert.Content>
            <Alert.Title>Impossible de charger vos organisations</Alert.Title>
            <Alert.Description>
              <Stack gap="3" align="flex-start">
                Une erreur est survenue lors du chargement de vos organisations.
                Veuillez réessayer.
                <Button
                  variant="outline"
                  size="sm"
                  colorPalette="red"
                  onClick={() => dispatch(fetchAllOrganizations())}
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

  if (!organizationsLoaded) {
    return (
      <Center h="100vh">
        <Spinner size="xl" color="blue.solid" />
      </Center>
    );
  }

  if (organizations.length === 0) {
    return <PrivateBetaPage />;
  }

  return <Outlet />;
};
