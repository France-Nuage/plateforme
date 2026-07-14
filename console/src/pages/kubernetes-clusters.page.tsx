import {
  Badge,
  Button,
  ButtonGroup,
  EmptyState,
  Flex,
  HStack,
  Heading,
  Spinner,
} from '@chakra-ui/react';
import {
  KubernetesCluster,
  KubernetesClusterHealthStatus,
} from '@france-nuage/sdk';
import { ColumnDef, createColumnHelper } from '@tanstack/react-table';
import { FunctionComponent, useEffect } from 'react';
import { HiPencil, HiPlus } from 'react-icons/hi';
import { HiServer } from 'react-icons/hi2';
import { Link } from 'react-router';

import { AppTable, DeleteKubernetesClusterButton } from '@/components';
import { fetchAllKubernetesClusters } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { Routes } from '@/types';

const columnHelper = createColumnHelper<KubernetesCluster>();

/**
 * Renders a health status badge with localized French label and color.
 */
const HealthStatusBadge: FunctionComponent<{
  status: KubernetesClusterHealthStatus;
}> = ({ status }) => {
  const label =
    status === KubernetesClusterHealthStatus.Healthy ? 'Sain' : 'Injoignable';
  const color =
    status === KubernetesClusterHealthStatus.Healthy ? 'green' : 'red';
  return (
    <Badge colorPalette={color} variant="subtle">
      {label}
    </Badge>
  );
};

// eslint-disable-next-line @typescript-eslint/no-explicit-any
const columns: ColumnDef<KubernetesCluster, any>[] = [
  columnHelper.accessor('name', {
    enableHiding: false,
    header: 'Nom',
    id: 'name',
  }),
  columnHelper.accessor('description', {
    header: 'Description',
    id: 'description',
  }),
  columnHelper.accessor('apiServerUrl', {
    header: 'Serveur API',
    id: 'apiServerUrl',
  }),
  columnHelper.accessor('healthStatus', {
    cell: ({ getValue }) => <HealthStatusBadge status={getValue()} />,
    header: 'Etat',
    id: 'healthStatus',
  }),
  columnHelper.accessor('labels', {
    cell: ({ getValue }) => (
      <Badge colorPalette="blue" variant="subtle">
        {getValue().length}
      </Badge>
    ),
    header: 'Labels',
    id: 'labels',
  }),
  columnHelper.accessor('lastHealthCheckAt', {
    cell: ({ getValue }) => {
      const v = getValue();
      return v ? new Date(v).toLocaleString() : '-';
    },
    header: 'Dernier controle',
    id: 'lastHealthCheckAt',
  }),
  columnHelper.accessor('createdAt', {
    cell: ({ getValue }) => new Date(getValue()).toLocaleString(),
    header: 'Date de creation',
    id: 'createdAt',
  }),
  columnHelper.display({
    cell: ({ row }) => (
      <ButtonGroup size="xs" variant="ghost">
        <Button asChild>
          <Link
            to={Routes.KubernetesClusterEdit.replace(
              ':clusterId',
              row.original.id,
            )}
          >
            <HiPencil />
          </Link>
        </Button>
        <DeleteKubernetesClusterButton cluster={row.original} />
      </ButtonGroup>
    ),
    header: 'Actions',
    id: 'actions',
  }),
];

/**
 * List page for Kubernetes clusters.
 *
 * Displays all clusters registered on the platform with health status badges.
 * Reserved for platform admins (guarded at the router level by AdminGuard).
 */
export const KubernetesClustersPage: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const clusters = useAppSelector((state) => state.kubernetesClusters.clusters);
  const loading = useAppSelector((state) => state.kubernetesClusters.loading);

  useEffect(() => {
    dispatch(fetchAllKubernetesClusters());
  }, [dispatch]);

  return (
    <>
      <HStack>
        <Heading flexGrow={1} whiteSpace="nowrap">
          Clusters Kubernetes
        </Heading>
        <Button asChild colorPalette="blue">
          <Link to={Routes.KubernetesClusterCreate}>
            <HiPlus />
            Nouveau cluster
          </Link>
        </Button>
      </HStack>

      {loading ? (
        <Flex justify="center" py={12}>
          <Spinner size="lg" />
        </Flex>
      ) : clusters.length === 0 ? (
        <EmptyState.Root>
          <EmptyState.Content>
            <EmptyState.Indicator>
              <HiServer />
            </EmptyState.Indicator>
            <EmptyState.Title>Aucun cluster enregistré</EmptyState.Title>
            <EmptyState.Description>
              Les clusters servent de cible de déploiement pour les utilisateurs
              de la plateforme. Ajoutez-en un pour commencer.
            </EmptyState.Description>
            <Button asChild colorPalette="blue">
              <Link to={Routes.KubernetesClusterCreate}>
                <HiPlus />
                Nouveau cluster
              </Link>
            </Button>
          </EmptyState.Content>
        </EmptyState.Root>
      ) : (
        <AppTable columns={columns} data={clusters} />
      )}
    </>
  );
};
