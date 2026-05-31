import {
  Button,
  Card,
  Field,
  Flex,
  HStack,
  Heading,
  Input,
  Stack,
  Text,
  Textarea,
} from '@chakra-ui/react';
import { FunctionComponent, useCallback, useState } from 'react';
import { HiArrowLeft } from 'react-icons/hi';
import { Link, useNavigate } from 'react-router';

import { createKubernetesCluster } from '@/features';
import { useAppDispatch } from '@/hooks';
import { Routes } from '@/types';
import { getErrorMessage } from '@/utils';

/**
 * Create form for a new Kubernetes cluster.
 *
 * Requires a name and a kubeconfig (YAML, write-only). Description is optional.
 * On submit, the backend performs a synchronous reachability health-check
 * (up to ~15 s) and rejects with a FAILED_PRECONDITION error when the cluster
 * is unreachable. The form surfaces that error clearly.
 *
 * Reserved for platform admins (guarded at the router level by AdminGuard).
 */
export const KubernetesClusterCreatePage: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [kubeconfig, setKubeconfig] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(() => {
    if (!name || !kubeconfig) return;

    setLoading(true);
    setError(null);

    dispatch(
      createKubernetesCluster({
        description: description || undefined,
        kubeconfig,
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
  }, [name, description, kubeconfig, dispatch, navigate]);

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

      <Heading>Nouveau cluster Kubernetes</Heading>

      <Card.Root>
        <Card.Body>
          <Stack gap={4}>
            <Field.Root required>
              <Field.Label>
                Nom
                <Field.RequiredIndicator />
              </Field.Label>
              <Input
                placeholder="mon-cluster-prod"
                value={name}
                onChange={(e) => setName(e.target.value)}
              />
            </Field.Root>

            <Field.Root>
              <Field.Label>Description</Field.Label>
              <Input
                placeholder="Description optionnelle"
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </Field.Root>

            <Field.Root required>
              <Field.Label>
                Kubeconfig
                <Field.RequiredIndicator />
              </Field.Label>
              <Field.HelperText>
                Collez le contenu YAML du kubeconfig. Ce champ est en écriture
                seule et ne sera jamais renvoyé par l'API.
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
                disabled={loading || !name || !kubeconfig}
                loading={loading}
                loadingText="Vérification du cluster en cours..."
                onClick={handleSubmit}
              >
                Créer le cluster
              </Button>
            </Flex>
          </Stack>
        </Card.Body>
      </Card.Root>
    </Stack>
  );
};
