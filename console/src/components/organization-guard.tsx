import { Center, Spinner } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Outlet } from 'react-router';

import { useAppSelector } from '@/hooks';
import { PrivateBetaPage } from '@/pages';

/**
 * Guard component that restricts access to users with organization membership.
 *
 * Displays a loading spinner while organizations are being fetched. Once loaded,
 * if the user has no organizations, the private beta page is shown instead of
 * the normal application layout. Users who belong to at least one organization
 * see the standard application content.
 */
export const OrganizationGuard: FunctionComponent = () => {
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const organizationsLoaded = useAppSelector(
    (state) => state.resources.organizationsLoaded,
  );

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
