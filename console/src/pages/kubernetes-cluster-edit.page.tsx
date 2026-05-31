import {
  Button,
  Card,
  Field,
  Flex,
  HStack,
  Heading,
  Input,
  Spinner,
  Stack,
  Text,
  Textarea,
} from '@chakra-ui/react';
import { FunctionComponent, useCallback, useEffect, useState } from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useNavigate, useParams } from 'react-router';

import {
  clearCurrentCluster,
  fetchKubernetesCluster,
  updateKubernetesCluster,
} from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';
import { Routes } from '@/types';
import { getErrorMessage } from '@/utils';

/**
 * Edit form for an existing Kubernetes cluster.
 *
 * Pre-fills name and description from the current cluster data. The kubeconfig
 * field is optional: leaving it empty keeps the existing kubeconfig on the
 * server. When a new kubeconfig is supplied, the backend performs the same
 * synchronous reachability check as on creation.
 *
 * Reserved for platform admins (guarded at the router level by AdminGuard).
 */
export const KubernetesClusterEditPage: FunctionComponent = () => {
  const { clusterId } = useParams<{ clusterId: string }>();
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const cluster = useAppSelector(
    (state) => state.kubernetesClusters.currentCluster,
  );

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [kubeconfig, setKubeconfig] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    dispatch(clearCurrentCluster());
    if (clusterId) {
      dispatch(fetchKubernetesCluster(clusterId));
    }
  }, [dispatch, clusterId]);

  // Pre-fill form once cluster data is loaded
  useEffect(() => {
    if (cluster) {
      setName(cluster.name);
      setDescription(cluster.description ?? '');
    }
  }, [cluster]);

  const handleSubmit = useCallback(() => {
    if (!clusterId || !name) return;

    setLoading(true);
    setError(null);

    dispatch(
      updateKubernetesCluster({
        clusterId,
        description: description || undefined,
        kubeconfig: kubeconfig || undefined,
        name,
      }),
    )
      .unwrap()
      .then(() => {
        setLoading(false);
        navigate(Routes.KubernetesClusters);
      })
      .catch((err: unknown) => {
        setError(getErrorMessage(err));
        setLoading(false);
      });
  }, [clusterId, name, description, kubeconfig, dispatch, navigate]);

  if (!cluster) {
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
          <Link to={Routes.KubernetesClusters}>
            <HiArrowLeft />
            Retour aux clusters
          </Link>
        </Button>
      </HStack>

      <Heading>Modifier {cluster.name}</Heading>

      <Card.Root>
        <Card.Body>
          <Stack gap={4}>
            <Field.Root required>
              <Field.Label>
                Nom
                <Field.RequiredIndicator />
              </Field.Label>
              <Input value={name} onChange={(e) => setName(e.target.value)} />
            </Field.Root>

            <Field.Root>
              <Field.Label>Description</Field.Label>
              <Input
                placeholder="Description optionnelle"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </Field.Root>

            <Field.Root>
              <Field.Label>Nouveau kubeconfig</Field.Label>
              <Field.HelperText>
                Laisser vide pour conserver le kubeconfig existant. Si
                renseigné, une vérification de connectivité sera effectuée par
                le serveur.
              </Field.HelperText>
              <Textarea
                placeholder="apiVersion: v1&#10;kind: Config&#10;..."
                value={kubeconfig}
                onChange={(e) => setKubeconfig(e.target.value)}
                rows={10}
                fontFamily="mono"
                fontSize="sm"
              />
            </Field.Root>

            {error && (
              <Text color="fg.error" fontSize="sm">
                {error}
              </Text>
            )}

            <Flex justify="end" gap={3}>
              <Button variant="outline" asChild disabled={loading}>
                <Link to={Routes.KubernetesClusters}>Annuler</Link>
              </Button>
              <Button
                colorPalette="blue"
                disabled={loading || !name}
                loading={loading}
                loadingText={
                  kubeconfig
                    ? 'Vérification du cluster en cours...'
                    : 'Enregistrement...'
                }
                onClick={handleSubmit}
              >
                Enregistrer
              </Button>
            </Flex>
          </Stack>
        </Card.Body>
      </Card.Root>
    </Stack>
  );
};
