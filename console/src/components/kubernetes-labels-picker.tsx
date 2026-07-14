import {
  Button,
  Field,
  HStack,
  Input,
  NativeSelect,
  Stack,
  Tag,
  Text,
  Wrap,
} from '@chakra-ui/react';
import { KubernetesLabel } from '@france-nuage/sdk';
import { FunctionComponent, useState } from 'react';

import { useKubernetesLabelRegistry } from '@/hooks';

/**
 * Validation of a label key or value, mirroring the backend rule: 1 to 49
 * chars, charset [a-zA-Z0-9-], starting and ending with an alphanumeric.
 */
const LABEL_PART_PATTERN = /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,47}[a-zA-Z0-9])?$/;

export type KubernetesLabelsPickerProps = {
  /** Currently selected labels. */
  value: KubernetesLabel[];
  /** Called with the new selection when the user adds or removes a label. */
  onChange: (labels: KubernetesLabel[]) => void;
  /** Disables every interaction (e.g. while the enclosing form submits). */
  disabled?: boolean;
};

/**
 * Selection of registry labels before a cluster exists.
 *
 * Unlike `ClusterLabelsEditor`, this is a fully controlled, local-only
 * picker: removing a label only updates the selection (nothing is attached
 * server-side yet), so no confirmation is needed. Creating a new key/value
 * pair persists the label in the registry immediately and adds it to the
 * selection.
 */
export const KubernetesLabelsPicker: FunctionComponent<
  KubernetesLabelsPickerProps
> = ({ value, onChange, disabled = false }) => {
  const { busy, createLabel, error, registryLabels } =
    useKubernetesLabelRegistry();

  const [selectedLabelId, setSelectedLabelId] = useState('');
  const [newKey, setNewKey] = useState('');
  const [newValue, setNewValue] = useState('');

  const availableLabels = registryLabels.filter(
    (label) =>
      !label.system && !value.some((selected) => selected.id === label.id),
  );
  const newPairIsValid =
    LABEL_PART_PATTERN.test(newKey) && LABEL_PART_PATTERN.test(newValue);

  const handleAddExisting = () => {
    const label = availableLabels.find(
      (candidate) => candidate.id === selectedLabelId,
    );
    if (label) {
      onChange([...value, label]);
      setSelectedLabelId('');
    }
  };

  const handleCreateAndAdd = () => {
    createLabel(newKey, newValue).then((label) => {
      if (label) {
        onChange([...value, label]);
        setNewKey('');
        setNewValue('');
      }
    });
  };

  return (
    <Stack gap={4}>
      <Wrap gap={2}>
        {value.map((label) => (
          <Tag.Root key={label.id} size="lg" colorPalette="blue">
            <Tag.Label fontFamily="mono">
              {`${label.key}=${label.value}`}
            </Tag.Label>
            <Tag.EndElement>
              <Tag.CloseTrigger
                aria-label={`Retirer ${label.key}=${label.value}`}
                disabled={disabled}
                onClick={() =>
                  onChange(value.filter((kept) => kept.id !== label.id))
                }
              />
            </Tag.EndElement>
          </Tag.Root>
        ))}
        {value.length === 0 && (
          <Text color="fg.muted" fontSize="sm">
            Aucun label sélectionné.
          </Text>
        )}
      </Wrap>

      {availableLabels.length > 0 && (
        <HStack>
          <NativeSelect.Root size="sm" maxW="280px" disabled={disabled}>
            <NativeSelect.Field
              aria-label="Label existant à ajouter"
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
            disabled={disabled || !selectedLabelId}
            onClick={handleAddExisting}
          >
            Ajouter
          </Button>
        </HStack>
      )}

      <HStack align="end" gap={3}>
        <Field.Root maxW="200px">
          <Field.Label>Clé</Field.Label>
          <Input
            size="sm"
            placeholder="availability"
            disabled={disabled}
            value={newKey}
            onChange={(event) => setNewKey(event.target.value)}
          />
        </Field.Root>
        <Field.Root maxW="200px">
          <Field.Label>Valeur</Field.Label>
          <Input
            size="sm"
            placeholder="ft"
            disabled={disabled}
            value={newValue}
            onChange={(event) => setNewValue(event.target.value)}
          />
        </Field.Root>
        <Button
          size="sm"
          disabled={disabled || !newPairIsValid || busy}
          loading={busy}
          onClick={handleCreateAndAdd}
        >
          Créer et ajouter
        </Button>
      </HStack>
      <Text color="fg.muted" fontSize="xs">
        49 caractères maximum, lettres, chiffres et tirets uniquement.
      </Text>

      {error && (
        <Text color="fg.error" fontSize="sm">
          {error}
        </Text>
      )}
    </Stack>
  );
};
