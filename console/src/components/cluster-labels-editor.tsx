import {
  Button,
  Dialog,
  Field,
  HStack,
  Input,
  NativeSelect,
  Portal,
  Spinner,
  Stack,
  Tag,
  Text,
  Wrap,
} from '@chakra-ui/react';
import { FunctionComponent, useState } from 'react';

import { useClusterLabels } from '@/hooks';

/**
 * Validation of a label key or value, mirroring the backend rule: 1 to 49
 * chars, charset [a-zA-Z0-9-], starting and ending with an alphanumeric.
 */
const LABEL_PART_PATTERN = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,47}[a-zA-Z0-9])?$/;

export type ClusterLabelsEditorProps = {
  /** ID of the cluster whose labels are edited. */
  clusterId: string;
};

/**
 * Editor of the labels attached to a Kubernetes cluster.
 *
 * Renders the attached labels as removable tags (system labels are
 * read-only), lets the operator attach an existing registry label or create
 * a new key/value pair, and guards every removal behind a confirmation
 * dialog listing the managed services whose deploy_target requires the
 * label. Removal is always allowed: running instances are unaffected, only
 * future deployments lose this cluster as a candidate.
 */
export const ClusterLabelsEditor: FunctionComponent<
  ClusterLabelsEditorProps
> = ({ clusterId }) => {
  const {
    attachedLabels,
    availableLabels,
    busy,
    error,
    detachConfirmation,
    attachExistingLabel,
    createAndAttachLabel,
    requestDetach,
    confirmDetach,
    cancelDetach,
  } = useClusterLabels(clusterId);

  const [selectedLabelId, setSelectedLabelId] = useState('');
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');

  const newPairIsValid =
    LABEL_PART_PATTERN.test(newKey) && LABEL_PART_PATTERN.test(newValue);

  const handleAttachExisting = () => {
    attachExistingLabel(selectedLabelId).then((succeeded) => {
      if (succeeded) {
        setSelectedLabelId('');
      }
    });
  };

  const handleCreateAndAttach = () => {
    createAndAttachLabel(newKey, newValue).then((succeeded) => {
      if (succeeded) {
        setNewKey('');
        setNewValue('');
      }
    });
  };

  return (
    <Stack gap={4}>
      <Wrap gap={2}>
        {attachedLabels.map((label) => (
          <Tag.Root
            key={label.id}
            size="lg"
            colorPalette={label.system ? 'gray' : 'blue'}
          >
            <Tag.Label fontFamily="mono">
              {`${label.key}=${label.value}`}
            </Tag.Label>
            {!label.system && (
              <Tag.EndElement>
                <Tag.CloseTrigger
                  aria-label={`Retirer ${label.key}=${label.value}`}
                  onClick={() => requestDetach(label)}
                />
              </Tag.EndElement>
            )}
          </Tag.Root>
        ))}
        {attachedLabels.length === 0 && (
          <Text color="fg.muted" fontSize="sm">
            Aucun label attaché à ce cluster.
          </Text>
        )}
      </Wrap>

      {availableLabels.length > 0 && (
        <HStack>
          <NativeSelect.Root size="sm" maxW="280px">
            <NativeSelect.Field
              aria-label="Label existant à attacher"
              value={selectedLabelId}
              onChange={(event) => setSelectedLabelId(event.target.value)}
            >
              <option value="">Choisir un label existant</option>
              {availableLabels.map((label) => (
                <option key={label.id} value={label.id}>
                  {`${label.key}=${label.value}`}
                </option>
              ))}
            </NativeSelect.Field>
            <NativeSelect.Indicator />
          </NativeSelect.Root>
          <Button
            size="sm"
            variant="outline"
            disabled={!selectedLabelId || busy}
            onClick={handleAttachExisting}
          >
            Attacher
          </Button>
        </HStack>
      )}

      <HStack align="end" gap={3}>
        <Field.Root maxW="200px">
          <Field.Label>Clé</Field.Label>
          <Input
            size="sm"
            placeholder="availability"
            value={newKey}
            onChange={(event) => setNewKey(event.target.value)}
          />
        </Field.Root>
        <Field.Root maxW="200px">
          <Field.Label>Valeur</Field.Label>
          <Input
            size="sm"
            placeholder="ft"
            value={newValue}
            onChange={(event) => setNewValue(event.target.value)}
          />
        </Field.Root>
        <Button
          size="sm"
          disabled={!newPairIsValid || busy}
          loading={busy}
          onClick={handleCreateAndAttach}
        >
          Créer et attacher
        </Button>
      </HStack>
      <Text color="fg.muted" fontSize="xs">
        49 caractères maximum, lettres, chiffres et tirets uniquement. Les
        labels système sont en lecture seule.
      </Text>

      {error && (
        <Text color="fg.error" fontSize="sm">
          {error}
        </Text>
      )}

      <Dialog.Root
        open={detachConfirmation.status !== 'idle'}
        onOpenChange={(event) => {
          if (!event.open && detachConfirmation.status !== 'detaching') {
            cancelDetach();
          }
        }}
      >
        <Portal>
          <Dialog.Backdrop />
          <Dialog.Positioner>
            <Dialog.Content>
              <Dialog.CloseTrigger />
              <Dialog.Header>
                <Dialog.Title>Retirer le label</Dialog.Title>
              </Dialog.Header>
              <Dialog.Body>
                {detachConfirmation.status === 'idle' ? null : (
                  <Stack gap={3}>
                    <Text>
                      Retirer le label{' '}
                      <strong>
                        {`${detachConfirmation.label.key}=${detachConfirmation.label.value}`}
                      </strong>{' '}
                      du cluster ?
                    </Text>
                    {detachConfirmation.status === 'loading' ? (
                      <HStack>
                        <Spinner size="sm" />
                        <Text fontSize="sm">
                          Vérification des services managés impactés...
                        </Text>
                      </HStack>
                    ) : detachConfirmation.services.length > 0 ? (
                      <Stack gap={2}>
                        <Text color="fg.warning" fontSize="sm">
                          Le déploiement des services managés suivants requiert
                          ce label. Les instances existantes ne sont pas
                          affectées, mais ce cluster ne sera plus éligible pour
                          leurs prochains déploiements :
                        </Text>
                        <Stack as="ul" gap={1} pl={5}>
                          {detachConfirmation.services.map((service) => (
                            <Text as="li" key={service.id} fontSize="sm">
                              {service.name}
                            </Text>
                          ))}
                        </Stack>
                      </Stack>
                    ) : (
                      <Text color="fg.muted" fontSize="sm">
                        Aucun service managé ne référence ce label.
                      </Text>
                    )}
                  </Stack>
                )}
              </Dialog.Body>
              <Dialog.Footer>
                <Button
                  variant="outline"
                  disabled={detachConfirmation.status === 'detaching'}
                  onClick={cancelDetach}
                >
                  Annuler
                </Button>
                <Button
                  colorPalette="red"
                  disabled={detachConfirmation.status !== 'ready'}
                  loading={detachConfirmation.status === 'detaching'}
                  loadingText="Retrait en cours..."
                  onClick={confirmDetach}
                >
                  Retirer
                </Button>
              </Dialog.Footer>
            </Dialog.Content>
          </Dialog.Positioner>
        </Portal>
      </Dialog.Root>
    </Stack>
  );
};
