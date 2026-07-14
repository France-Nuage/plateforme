import {
  Badge,
  Box,
  Button,
  Card,
  Flex,
  HStack,
  Heading,
  IconButton,
  Spinner,
  Stack,
  Text,
} from '@chakra-ui/react';
import { ConnectionInfoField, ManagedInstanceStatus } from '@france-nuage/sdk';
import { FunctionComponent, useEffect, useMemo, useState } from 'react';
import { HiArrowLeft, HiClipboard, HiEye, HiEyeOff } from 'react-icons/hi';
import { Link, useNavigate, useParams } from 'react-router';

import {
  DeleteManagedInstanceButton,
  ManagedInstanceStatusBadge,
  UpgradeManagedInstanceButton,
} from '@/components';
import {
  fetchInstanceConnectionInfo,
  fetchManagedInstance,
  fetchManagedServicePlans,
  fetchManagedServiceVersions,
} from '@/features';
import { useAppDispatch, useAppSelector, usePolling } from '@/hooks';
import { MANAGED_INSTANCE_POLL_INTERVAL_MS, parseJsonObject } from '@/services';
import { Routes } from '@/types';

/**
 * Detail page for a managed service instance.
 *
 * Shows release metadata, current chart/app version and the rendered
 * user configuration. Exposes upgrade and delete actions, gated by the
 * current status. Polls the instance to surface transitions.
 */
export const ManagedInstanceDetailPage: FunctionComponent = () => {
  const { instanceId } = useParams<{ instanceId: string }>();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const instance = useAppSelector(
    (state) => state.managedServices.currentInstance,
  );
  const services = useAppSelector((state) => state.managedServices.services);
  const versions = useAppSelector((state) => state.managedServices.versions);
  const plans = useAppSelector((state) => state.managedServices.plans);
  const connectionInfo = useAppSelector(
    (state) => state.managedServices.connectionInfo,
  );
  const connectionInfoLoading = useAppSelector(
    (state) => state.managedServices.connectionInfoLoading,
  );

  useEffect(() => {
    if (instanceId) {
      dispatch(fetchManagedInstance(instanceId));
    }
  }, [dispatch, instanceId]);

  usePolling(
    () => {
      if (instanceId) {
        dispatch(fetchManagedInstance(instanceId));
      }
    },
    MANAGED_INSTANCE_POLL_INTERVAL_MS,
    !!instanceId,
  );

  useEffect(() => {
    if (!instance) return;
    const service = services.find(
      (service) => service.id === instance.serviceId,
    );
    if (service) {
      dispatch(fetchManagedServiceVersions(service.slug));
      dispatch(fetchManagedServicePlans(service.slug));
    }
  }, [dispatch, instance, services]);

  const instanceStatus = instance?.status;
  useEffect(() => {
    if (instanceId && instanceStatus === ManagedInstanceStatus.Running) {
      dispatch(fetchInstanceConnectionInfo(instanceId));
    }
  }, [dispatch, instanceId, instanceStatus]);

  const serviceName = useMemo(
    () =>
      services.find((service) => service.id === instance?.serviceId)?.name ??
      instance?.serviceId ??
      '',
    [services, instance],
  );

  const currentVersion = useMemo(
    () => versions.find((version) => version.id === instance?.versionId),
    [versions, instance],
  );

  const planName = useMemo(
    () => plans.find((plan) => plan.id === instance?.planId)?.name,
    [plans, instance],
  );

  const userValues = useMemo(
    () => parseJsonObject(instance?.userValues),
    [instance],
  );

  if (!instance) {
    return (
      <Flex justify="center" py={12}>
        <Spinner size="lg" />
      </Flex>
    );
  }

  const canUpgrade = instance.status === ManagedInstanceStatus.Running;
  const canDelete = instance.status === ManagedInstanceStatus.Running;

  return (
    <Stack gap={6}>
      <HStack>
        <Button variant="ghost" size="sm" asChild>
          <Link to={Routes.ManagedInstances}>
            <HiArrowLeft />
            Retour aux instances
          </Link>
        </Button>
      </HStack>

      <Flex justify="space-between" align="start" wrap="wrap" gap={4}>
        <Stack gap={2}>
          <Heading size="2xl">{instance.releaseName}</Heading>
          <HStack gap={2}>
            <Badge variant="subtle">{serviceName}</Badge>
            <ManagedInstanceStatusBadge status={instance.status} />
          </HStack>
        </Stack>
        <HStack gap={2}>
          {canUpgrade && (
            <UpgradeManagedInstanceButton
              instance={instance}
              versions={versions}
            />
          )}
          {canDelete && (
            <DeleteManagedInstanceButton
              instance={instance}
              label="Supprimer"
              onDeleted={() => navigate(Routes.ManagedInstances)}
            />
          )}
        </HStack>
      </Flex>

      <Flex gap={4} direction={{ base: 'column', lg: 'row' }}>
        <Card.Root flex={1}>
          <Card.Header>
            <Card.Title>Informations</Card.Title>
          </Card.Header>
          <Card.Body>
            <Stack gap={2}>
              <InfoRow label="ID" value={instance.id} mono />
              <InfoRow label="Namespace" value={instance.namespace} mono />
              <InfoRow label="Release" value={instance.releaseName} mono />
              {planName && <InfoRow label="Plan" value={planName} />}
              <InfoRow
                label="Version chart"
                value={currentVersion?.chartVersion ?? instance.versionId}
                mono
              />
              {currentVersion?.appVersion && (
                <InfoRow
                  label="Version application"
                  value={currentVersion.appVersion}
                  mono
                />
              )}
              <InfoRow
                label="Date de création"
                value={new Date(instance.createdAt).toLocaleString()}
              />
            </Stack>
          </Card.Body>
        </Card.Root>

        {userValues && (
          <Card.Root flex={1}>
            <Card.Header>
              <Card.Title>Configuration</Card.Title>
            </Card.Header>
            <Card.Body>
              <Box
                as="pre"
                fontSize="xs"
                bg="bg.subtle"
                p={3}
                borderRadius="md"
                overflow="auto"
                maxH="300px"
              >
                {JSON.stringify(userValues, null, 2)}
              </Box>
            </Card.Body>
          </Card.Root>
        )}
      </Flex>

      {connectionInfoLoading && (
        <Flex justify="center" py={4}>
          <Spinner size="sm" />
        </Flex>
      )}

      {connectionInfo.length > 0 && (
        <Card.Root>
          <Card.Header>
            <Card.Title>Informations de connexion</Card.Title>
          </Card.Header>
          <Card.Body>
            <Stack gap={2}>
              {connectionInfo.map((field) => (
                <ConnectionInfoRow key={field.key} field={field} />
              ))}
            </Stack>
          </Card.Body>
        </Card.Root>
      )}
    </Stack>
  );
};

const InfoRow: FunctionComponent<{
  label: string;
  value: string;
  mono?: boolean;
}> = ({ label, value, mono }) => (
  <Flex justify="space-between" align="center">
    <Text fontSize="sm" color="fg.muted">
      {label}
    </Text>
    <Text
      fontSize="sm"
      fontFamily={mono ? 'mono' : undefined}
      fontWeight="medium"
    >
      {value}
    </Text>
  </Flex>
);

const ConnectionInfoRow: FunctionComponent<{
  field: ConnectionInfoField;
}> = ({ field }) => {
  const [visible, setVisible] = useState(false);

  const copyToClipboard = () => {
    navigator.clipboard.writeText(field.value);
  };

  return (
    <Flex justify="space-between" align="center">
      <Text fontSize="sm" color="fg.muted">
        {field.label}
      </Text>
      <HStack gap={1}>
        <Text fontSize="sm" fontFamily="mono" fontWeight="medium">
          {field.display === 'password' && !visible ? '********' : field.value}
        </Text>
        {field.display === 'password' && (
          <IconButton
            aria-label={visible ? 'Masquer' : 'Afficher'}
            size="xs"
            variant="ghost"
            onClick={() => setVisible(!visible)}
          >
            {visible ? <HiEyeOff /> : <HiEye />}
          </IconButton>
        )}
        {(field.display === 'copy' || field.display === 'password') && (
          <IconButton
            aria-label="Copier"
            size="xs"
            variant="ghost"
            onClick={copyToClipboard}
          >
            <HiClipboard />
          </IconButton>
        )}
      </HStack>
    </Flex>
  );
};
