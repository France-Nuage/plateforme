import {
  Badge,
  Button,
  Card,
  Flex,
  Grid,
  HStack,
  Heading,
  Spinner,
  Stack,
  Table,
  Text,
} from '@chakra-ui/react';
import {
  ManagedServicePlan,
  ManagedServiceVersion,
} from '@france-nuage/sdk';
import { FunctionComponent, useEffect, useMemo, useState } from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useParams } from 'react-router';

import {
  fetchManagedService,
  fetchManagedServicePlans,
  fetchManagedServiceVersions,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { MANAGED_SERVICE_CATEGORY_LABELS } from '@/services';

function formatPrice(cents: number): string {
  return (cents / 100).toFixed(2).replace('.', ',');
}

/**
 * Detail page for a managed service: presentation, plans, and chart version table.
 * Triggers a deploy flow via the "Deploy" button.
 */
export const ManagedServiceDetailPage: FunctionComponent = () => {
  const { slug } = useParams<{ slug: string }>();
  const dispatch = useAppDispatch();
  const service = useAppSelector(
    (state) => state.managedServices.currentService,
  );
  const versions = useAppSelector((state) => state.managedServices.versions);
  const plans = useAppSelector((state) => state.managedServices.plans);

  useEffect(() => {
    if (slug) {
      dispatch(fetchManagedService(slug));
      dispatch(fetchManagedServiceVersions(slug));
      dispatch(fetchManagedServicePlans(slug));
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
            Deployer une instance
          </Link>
        </Button>
      </Flex>

      {plans.length > 0 && (
        <>
          <Heading size="lg">Plans</Heading>
          <PlansGrid plans={plans} />
        </>
      )}

      <Heading size="lg">Versions</Heading>
      <VersionsTable versions={versions} />
    </Stack>
  );
};

const PlansGrid: FunctionComponent<{ plans: ManagedServicePlan[] }> = ({
  plans,
}) => {
  const [billingPeriod, setBillingPeriod] = useState<'monthly' | 'yearly'>(
    'monthly',
  );

  return (
    <Stack gap={3}>
      <HStack gap={2}>
        <Text
          fontSize="sm"
          cursor="pointer"
          fontWeight={billingPeriod === 'monthly' ? 'bold' : 'normal'}
          onClick={() => setBillingPeriod('monthly')}
        >
          Mensuel
        </Text>
        <Text fontSize="sm" color="fg.muted">
          |
        </Text>
        <Text
          fontSize="sm"
          cursor="pointer"
          fontWeight={billingPeriod === 'yearly' ? 'bold' : 'normal'}
          onClick={() => setBillingPeriod('yearly')}
        >
          Annuel
        </Text>
      </HStack>
      <Grid
        templateColumns={{
          base: '1fr',
          md: `repeat(${Math.min(plans.length, 3)}, 1fr)`,
        }}
        gap={4}
      >
        {plans.map((plan) => (
          <PlanReadonlyCard
            key={plan.id}
            plan={plan}
            billingPeriod={billingPeriod}
          />
        ))}
      </Grid>
    </Stack>
  );
};

const PlanReadonlyCard: FunctionComponent<{
  plan: ManagedServicePlan;
  billingPeriod: 'monthly' | 'yearly';
}> = ({ plan, billingPeriod }) => {
  const displayPrice = useMemo(() => {
    const cents =
      billingPeriod === 'monthly'
        ? plan.priceMonthlyCents
        : plan.priceYearlyCents;
    if (cents === undefined) return null;
    return formatPrice(cents);
  }, [plan.priceMonthlyCents, plan.priceYearlyCents, billingPeriod]);

  return (
    <Card.Root
      borderWidth={plan.highlighted ? '2px' : '1px'}
      borderColor={plan.highlighted ? 'blue.500' : 'border'}
    >
      <Card.Body>
        <Stack gap={3}>
          <Flex justify="space-between" align="start">
            <Heading size="md">{plan.name}</Heading>
            {plan.highlighted && (
              <Badge colorPalette="blue" variant="solid" size="sm">
                Recommande
              </Badge>
            )}
          </Flex>

          {plan.description && (
            <Text fontSize="sm" color="fg.muted">
              {plan.description}
            </Text>
          )}

          {plan.entitlements.length > 0 && (
            <HStack gap={2} flexWrap="wrap">
              {plan.entitlements.map((e) => (
                <Badge key={e.key} variant="outline" size="sm">
                  {e.label}: {e.value}
                </Badge>
              ))}
            </HStack>
          )}

          {displayPrice !== null && (
            <Text fontSize="2xl" fontWeight="bold">
              {displayPrice} EUR
              <Text as="span" fontSize="sm" fontWeight="normal" color="fg.muted">
                {' '}
                /{billingPeriod === 'monthly' ? 'mois' : 'an'}
              </Text>
            </Text>
          )}
        </Stack>
      </Card.Body>
    </Card.Root>
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
          <Table.ColumnHeader>Reference OCI</Table.ColumnHeader>
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
