import {
  Badge,
  Button,
  Flex,
  HStack,
  Heading,
  Spinner,
  Stack,
  Table,
  Text,
} from '@chakra-ui/react';
import { ManagedServiceVersion } from '@france-nuage/sdk';
import { FunctionComponent, useEffect } from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useParams } from 'react-router';

import { fetchManagedService, fetchManagedServiceVersions } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { MANAGED_SERVICE_CATEGORY_LABELS } from '@/services';

/**
 * Detail page for a managed service: presentation and chart version table.
 * Triggers a deploy flow via the "Deploy" button.
 */
export const ManagedServiceDetailPage: FunctionComponent = () => {
  const { slug } = useParams<{ slug: string }>();
  const dispatch = useAppDispatch();
  const service = useAppSelector(
    (state) => state.managedServices.currentService,
  );
  const versions = useAppSelector((state) => state.managedServices.versions);

  useEffect(() => {
    if (slug) {
      dispatch(fetchManagedService(slug));
      dispatch(fetchManagedServiceVersions(slug));
    }
  }, [dispatch, slug]);

  if (!service) {
    return (
      <Flex justify="center" py={12}>
        <Spinner size="lg" />
      </Flex>
    );
  }

  return (
    <Stack gap={6}>
      <HStack>
        <Button variant="ghost" size="sm" asChild>
          <Link to="/managed-services">
            <HiArrowLeft />
            Retour au catalogue
          </Link>
        </Button>
      </HStack>

      <Flex justify="space-between" align="start" wrap="wrap" gap={4}>
        <Stack gap={2}>
          <Heading size="2xl">{service.name}</Heading>
          <HStack gap={2}>
            <Badge variant="subtle">
              {MANAGED_SERVICE_CATEGORY_LABELS[service.category] ??
                service.category}
            </Badge>
            {service.databaseEngine && (
              <Badge variant="outline">{service.databaseEngine}</Badge>
            )}
          </HStack>
          {service.description && (
            <Text color="fg.muted" maxW="640px">
              {service.description}
            </Text>
          )}
        </Stack>
        <Button colorPalette="blue" asChild>
          <Link to={`/managed-services/${slug}/deploy`}>
            Déployer une instance
          </Link>
        </Button>
      </Flex>

      <Heading size="lg">Versions</Heading>
      <VersionsTable versions={versions} />
    </Stack>
  );
};

const VersionsTable: FunctionComponent<{
  versions: ManagedServiceVersion[];
}> = ({ versions }) => {
  if (versions.length === 0) {
    return <Text color="fg.muted">Aucune version disponible.</Text>;
  }

  return (
    <Table.Root size="sm" variant="outline">
      <Table.Header>
        <Table.Row>
          <Table.ColumnHeader>Version chart</Table.ColumnHeader>
          <Table.ColumnHeader>Version application</Table.ColumnHeader>
          <Table.ColumnHeader>Référence OCI</Table.ColumnHeader>
          <Table.ColumnHeader>Date</Table.ColumnHeader>
        </Table.Row>
      </Table.Header>
      <Table.Body>
        {versions.map((version) => (
          <Table.Row key={version.id}>
            <Table.Cell fontFamily="mono">{version.chartVersion}</Table.Cell>
            <Table.Cell fontFamily="mono">
              {version.appVersion ?? '-'}
            </Table.Cell>
            <Table.Cell fontSize="xs" fontFamily="mono">
              {version.ociReference}
            </Table.Cell>
            <Table.Cell>
              {new Date(version.createdAt).toLocaleDateString()}
            </Table.Cell>
          </Table.Row>
        ))}
      </Table.Body>
    </Table.Root>
  );
};
