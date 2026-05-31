import {
  Box,
  Button,
  Card,
  Flex,
  HStack,
  Heading,
  Select,
  Spinner,
  Stack,
  Text,
  useListCollection,
} from '@chakra-ui/react';
import {
  FunctionComponent,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useNavigate, useParams } from 'react-router';

import { RjsfDeployForm } from '@/components';
import {
  createManagedInstance,
  fetchManagedService,
  fetchManagedServiceVersions,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { parseJsonObject } from '@/services';
import { Routes } from '@/types';
import { getErrorMessage } from '@/utils';

/**
 * Form page deploying a new instance of a managed service.
 *
 * The form is fed by the service catalog versions. Default selection is
 * computed from the current data via `useMemo` rather than `useEffect`,
 * keeping the UI deterministic.
 */
export const ManagedServiceDeployPage: FunctionComponent = () => {
  const { slug } = useParams<{ slug: string }>();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const service = useAppSelector(
    (state) => state.managedServices.currentService,
  );
  const versions = useAppSelector((state) => state.managedServices.versions);
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization,
  );

  useEffect(() => {
    if (slug) {
      dispatch(fetchManagedService(slug));
      dispatch(fetchManagedServiceVersions(slug));
    }
  }, [dispatch, slug]);

  const versionItems = useMemo(
    () =>
      versions.map((version) => ({
        label: `${version.chartVersion}${version.appVersion ? ` (app: ${version.appVersion})` : ''}`,
        value: version.id,
      })),
    [versions],
  );

  const defaultVersionId = versions[0]?.id ?? '';

  const [selectedVersionId, setSelectedVersionId] = useState<string[]>([]);
  const [userValues, setUserValues] = useState('');
  const [secretValues, setSecretValues] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

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

  const handleDeploy = useCallback(() => {
    if (!slug || !activeProject || !activeOrganization || !effectiveVersionId) {
      return;
    }

    setLoading(true);
    setError(null);

    dispatch(
      createManagedInstance({
        organizationId: activeOrganization.id,
        projectId: activeProject.id,
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
  }, [
    slug,
    activeProject,
    activeOrganization,
    effectiveVersionId,
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

  return (
    <Stack gap={6} maxW="640px">
      <HStack>
        <Button variant="ghost" size="sm" asChild>
          <Link to={`/managed-services/${slug}`}>
            <HiArrowLeft />
            Retour à {service.name}
          </Link>
        </Button>
      </HStack>

      <Heading>Déployer {service.name}</Heading>

      <Card.Root>
        <Card.Body>
          <Stack gap={4}>
            <DeployFormSelect
              label="Version"
              placeholder="Sélectionner une version"
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
              <Text color="fg.error" fontSize="sm">
                {error}
              </Text>
            )}

            <Flex justify="end">
              <Button
                colorPalette="blue"
                disabled={loading || !effectiveVersionId}
                loading={loading}
                loadingText="Déploiement en cours..."
                onClick={handleDeploy}
              >
                Déployer
              </Button>
            </Flex>
          </Stack>
        </Card.Body>
      </Card.Root>
    </Stack>
  );
};

type SelectItem = { label: string; value: string };

const DeployFormSelect: FunctionComponent<{
  label: string;
  placeholder: string;
  items: SelectItem[];
  collection: ReturnType<typeof useListCollection<SelectItem>>['collection'];
  value: string[];
  onValueChange: (value: string[]) => void;
}> = ({ label, placeholder, items, collection, value, onValueChange }) => (
  <Box>
    <Text fontWeight="medium" mb={1}>
      {label}
    </Text>
    <Select.Root
      collection={collection}
      value={value}
      onValueChange={(event) => onValueChange(event.value)}
    >
      <Select.Control>
        <Select.Trigger>
          <Select.ValueText placeholder={placeholder} />
        </Select.Trigger>
        <Select.IndicatorGroup>
          <Select.Indicator />
        </Select.IndicatorGroup>
      </Select.Control>
      <Select.Positioner>
        <Select.Content>
          {items.map((item) => (
            <Select.Item item={item} key={item.value}>
              {item.label}
              <Select.ItemIndicator />
            </Select.Item>
          ))}
        </Select.Content>
      </Select.Positioner>
    </Select.Root>
  </Box>
);
