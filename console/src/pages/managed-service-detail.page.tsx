import {
  Alert,
  Badge,
  Box,
  Button,
  Card,
  Flex,
  Grid,
  HStack,
  Heading,
  SegmentGroup,
  Select,
  Separator,
  Spinner,
  Stack,
  Text,
  useListCollection,
} from '@chakra-ui/react';
import { parseAsString, parseAsStringLiteral, useQueryStates } from 'nuqs';
import {
  FunctionComponent,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useNavigate, useParams } from 'react-router';

import { ManagedServicePlanCard, RjsfDeployForm } from '@/components';
import {
  createCheckoutSession,
  createManagedInstance,
  fetchManagedService,
  fetchManagedServicePlans,
  fetchManagedServiceVersions,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import {
  MANAGED_SERVICE_CATEGORY_COLORS,
  MANAGED_SERVICE_CATEGORY_LABELS,
  parseJsonObject,
} from '@/services';
import { Routes } from '@/types';
import { getErrorMessage } from '@/utils';

export const ManagedServiceDetailPage: FunctionComponent = () => {
  const { slug } = useParams<{ slug: string }>();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const service = useAppSelector(
    (state) => state.managedServices.currentService,
  );
  const versions = useAppSelector((state) => state.managedServices.versions);
  const plans = useAppSelector((state) => state.managedServices.plans);
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization,
  );

  const [{ billing: billingPeriod, plan: selectedPlanId }, setDeployParams] =
    useQueryStates({
      billing: parseAsStringLiteral(['monthly', 'yearly'] as const).withDefault(
        'monthly',
      ),
      plan: parseAsString,
    });
  const setSelectedPlanId = useCallback(
    (id: string) => setDeployParams({ plan: id }),
    [setDeployParams],
  );
  const setBillingPeriod = useCallback(
    (value: 'monthly' | 'yearly') => setDeployParams({ billing: value }),
    [setDeployParams],
  );
  const [selectedVersionId, setSelectedVersionId] = useState<string[]>([]);
  const [userValues, setUserValues] = useState('');
  const [secretValues, setSecretValues] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (slug) {
      dispatch(fetchManagedService(slug));
      dispatch(fetchManagedServiceVersions(slug));
      dispatch(fetchManagedServicePlans(slug));
    }
  }, [dispatch, slug]);

  const versionItems = useMemo(
    () =>
      versions.map((version, index) => ({
        label: `${version.chartVersion}${version.appVersion ? ` (app: ${version.appVersion})` : ''}`,
        latest: index === 0,
        value: version.id,
      })),
    [versions],
  );

  const defaultVersionId = versions[0]?.id ?? '';

  const effectiveVersionId =
    selectedVersionId[0] ?? (defaultVersionId ? defaultVersionId : undefined);

  const { collection: versionCollection } = useListCollection({
    initialItems: versionItems,
  });

  const selectedVersion = useMemo(
    () => versions.find((version) => version.id === effectiveVersionId),
    [versions, effectiveVersionId],
  );

  const valuesSchema = useMemo(
    () => parseJsonObject(selectedVersion?.configurableValuesSchema),
    [selectedVersion],
  );

  const uiSchema = useMemo(
    () => parseJsonObject(selectedVersion?.uiSchema),
    [selectedVersion],
  );

  const handleRjsfChange = useCallback(
    ({
      userValues: next,
      secretValues: nextSecret,
    }: {
      userValues: string;
      secretValues: string;
    }) => {
      setUserValues(next);
      setSecretValues(nextSecret);
    },
    [],
  );

  const selectedPlan = useMemo(
    () => plans.find((p) => p.id === selectedPlanId),
    [plans, selectedPlanId],
  );

  const handleDeploy = useCallback(() => {
    if (
      !slug ||
      !activeProject ||
      !activeOrganization ||
      !effectiveVersionId ||
      !selectedPlanId ||
      !selectedPlan
    ) {
      return;
    }

    setLoading(true);
    setError(null);

    if (selectedPlan.requiresPayment) {
      dispatch(
        createCheckoutSession({
          billingPeriod,
          organizationSlug: activeOrganization.slug,
          planId: selectedPlanId,
          projectSlug: activeProject.slug,
          secretValues: secretValues || undefined,
          serviceSlug: slug,
          userValues: userValues || undefined,
          versionId: effectiveVersionId,
        }),
      )
        .unwrap()
        .then((result) => {
          window.location.href = result.checkoutUrl;
        })
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setLoading(false);
        });
    } else {
      dispatch(
        createManagedInstance({
          organizationSlug: activeOrganization.slug,
          planId: selectedPlanId,
          projectSlug: activeProject.slug,
          secretValues: secretValues || undefined,
          serviceSlug: slug,
          userValues: userValues || undefined,
          versionId: effectiveVersionId,
        }),
      )
        .unwrap()
        .then(() => navigate(Routes.ManagedInstances))
        .catch((err: unknown) => {
          setError(getErrorMessage(err));
          setLoading(false);
        });
    }
  }, [
    slug,
    activeProject,
    activeOrganization,
    effectiveVersionId,
    selectedPlanId,
    selectedPlan,
    billingPeriod,
    userValues,
    secretValues,
    dispatch,
    navigate,
  ]);

  if (!service) {
    return (
      <Flex justify="center" py={12}>
        <Spinner size="lg" />
      </Flex>
    );
  }

  const categoryColor = MANAGED_SERVICE_CATEGORY_COLORS[service.category];

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

      <Stack gap={2}>
        <Heading size="2xl">{service.name}</Heading>
        <HStack gap={2}>
          <Badge variant="subtle" colorPalette={categoryColor}>
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

      {plans.length > 0 && (
        <Stack gap={3}>
          <Flex justify="space-between" align="center">
            <Heading size="md">Plan</Heading>
            <SegmentGroup.Root
              size="sm"
              value={billingPeriod}
              onValueChange={(e) =>
                setBillingPeriod(e.value as 'monthly' | 'yearly')
              }
            >
              <SegmentGroup.Indicator />
              <SegmentGroup.Item value="monthly">
                <SegmentGroup.ItemText>Mensuel</SegmentGroup.ItemText>
                <SegmentGroup.ItemHiddenInput />
              </SegmentGroup.Item>
              <SegmentGroup.Item value="yearly">
                <SegmentGroup.ItemText>Annuel</SegmentGroup.ItemText>
                <SegmentGroup.ItemHiddenInput />
              </SegmentGroup.Item>
            </SegmentGroup.Root>
          </Flex>
          <Grid
            templateColumns={{
              base: '1fr',
              md: `repeat(${Math.min(plans.length, 3)}, 1fr)`,
            }}
            gap={4}
          >
            {plans.map((plan) => (
              <ManagedServicePlanCard
                key={plan.id}
                plan={plan}
                billingPeriod={billingPeriod}
                selected={selectedPlanId === plan.id}
                onSelect={() => setSelectedPlanId(plan.id)}
              />
            ))}
          </Grid>
        </Stack>
      )}

      <Separator />

      <Card.Root>
        <Card.Body>
          <Stack gap={4}>
            <VersionSelect
              placeholder="Selectionner une version"
              items={versionItems}
              collection={versionCollection}
              value={
                selectedVersionId.length
                  ? selectedVersionId
                  : [defaultVersionId]
              }
              onValueChange={setSelectedVersionId}
            />

            {valuesSchema && (
              <RjsfDeployForm
                schema={valuesSchema}
                uiSchema={uiSchema ?? undefined}
                userValues={userValues}
                secretValues={secretValues}
                onValuesChange={handleRjsfChange}
              />
            )}

            {error && (
              <Alert.Root status="error">
                <Alert.Indicator />
                <Alert.Content>
                  <Alert.Title>Deploiement impossible</Alert.Title>
                  <Alert.Description>{error}</Alert.Description>
                </Alert.Content>
              </Alert.Root>
            )}

            <Flex justify="end">
              <Button
                colorPalette="blue"
                disabled={loading || !effectiveVersionId || !selectedPlanId}
                loading={loading}
                loadingText="Deploiement en cours..."
                onClick={handleDeploy}
              >
                Deployer
              </Button>
            </Flex>
          </Stack>
        </Card.Body>
      </Card.Root>
    </Stack>
  );
};

type VersionItem = { label: string; value: string; latest: boolean };

const VersionSelect: FunctionComponent<{
  placeholder: string;
  items: VersionItem[];
  collection: ReturnType<typeof useListCollection<VersionItem>>['collection'];
  value: string[];
  onValueChange: (value: string[]) => void;
}> = ({ placeholder, items, collection, value, onValueChange }) => {
  const selectedItem = items.find((i) => value.includes(i.value));

  return (
    <Box>
      <Text fontWeight="medium" mb={1}>
        Version
      </Text>
      <Select.Root
        collection={collection}
        value={value}
        onValueChange={(event) => onValueChange(event.value)}
      >
        <Select.Control>
          <Select.Trigger>
            {selectedItem ? (
              <HStack gap={2}>
                <Text>{selectedItem.label}</Text>
                {selectedItem.latest && (
                  <Badge colorPalette="green" variant="subtle" size="sm">
                    Derniere version
                  </Badge>
                )}
              </HStack>
            ) : (
              <Select.ValueText placeholder={placeholder} />
            )}
          </Select.Trigger>
          <Select.IndicatorGroup>
            <Select.Indicator />
          </Select.IndicatorGroup>
        </Select.Control>
        <Select.Positioner>
          <Select.Content>
            {items.map((item) => (
              <Select.Item item={item} key={item.value}>
                <HStack gap={2}>
                  <Text>{item.label}</Text>
                  {item.latest && (
                    <Badge colorPalette="green" variant="subtle" size="sm">
                      Derniere version
                    </Badge>
                  )}
                </HStack>
                <Select.ItemIndicator />
              </Select.Item>
            ))}
          </Select.Content>
        </Select.Positioner>
      </Select.Root>
    </Box>
  );
};
