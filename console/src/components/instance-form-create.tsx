import {
  Button,
  Field,
  HStack,
  Heading,
  Input,
  NativeSelect,
  RadioCard,
  Switch,
  VStack,
} from '@chakra-ui/react';
import { InstanceFormValue, Project } from '@france-nuage/sdk';
import { FunctionComponent, useState } from 'react';
import { useNavigate } from 'react-router';

import { createInstance } from '@/features';
import { useAppDispatch } from '@/hooks';
import { bytesToGB } from '@/services';
import { Routes } from '@/types';

import { AppCodeEditor } from './app-code-editor';

export type InstanceFormProps = {
  projects: Project[];
};

type InstanceType = {
  cpuCores: number;
  diskBytes: number;
  memoryBytes: number;
  name: string;
};

const choices: InstanceType[] = [
  {
    cpuCores: 1,
    diskBytes: 10 * 1024 ** 3,
    memoryBytes: 1 * 1024 ** 3,
    name: 'XS',
  },
  {
    cpuCores: 2,
    diskBytes: 30 * 1024 ** 3,
    memoryBytes: 4 * 1024 ** 3,
    name: 'M',
  },
  {
    cpuCores: 4,
    diskBytes: 50 * 1024 ** 3,
    memoryBytes: 8 * 1024 ** 3,
    name: 'XL',
  },
];

export const InstanceForm: FunctionComponent<InstanceFormProps> = ({
  projects,
}) => {
  const dispatch = useAppDispatch();
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [isSnippetEditable, setSnippetEditable] = useState(false);
  const [instanceType, setInstanceType] = useState<InstanceType>(choices[0]);

  const [formValue, setFormValue] = useState<InstanceFormValue>({
    image: 'debian-12-genericcloud-amd64-20250316-2053.qcow2',
    maxCpuCores: instanceType.cpuCores,
    maxDiskBytes: instanceType.diskBytes,
    maxMemoryBytes: instanceType.memoryBytes,
    name: '',
    projectId: projects[0]?.id,
    snippet: foo,
  });

  const handleInstanceTypeChange = (newValue: InstanceType) => {
    setInstanceType(newValue);
    setFormValue({
      ...formValue,
      maxCpuCores: newValue.cpuCores,
      maxDiskBytes: newValue.diskBytes,
      maxMemoryBytes: newValue.memoryBytes,
    });
  };

  const handleSubmit = async () => {
    setLoading(true);
    dispatch(createInstance(formValue))
      .then(() => navigate(Routes.Instances))
      .finally(() => setLoading(false));
  };

  return (
    <VStack alignItems="start" gapY={4}>
      <HStack justifyContent="space-between">
        <Heading>Nouvelle instance</Heading>
        <Button
          onClick={handleSubmit}
          loading={loading}
          loadingText="Création de l'instance en cours..."
        >
          Créer la nouvelle instance
        </Button>
      </HStack>

      {/* name */}
      <Field.Root disabled={loading}>
        <Field.Label>
          Nom de l'instance
          <Field.RequiredIndicator />
        </Field.Label>
        <Input
          onChange={(event) =>
            setFormValue({ ...formValue, name: event.currentTarget.value })
          }
          value={formValue.name}
        />
      </Field.Root>

      {/* project */}
      <Field.Root disabled={loading}>
        <Field.Label>
          Projet
          <Field.RequiredIndicator />
        </Field.Label>
        <NativeSelect.Root>
          <NativeSelect.Field
            onChange={(event) =>
              setFormValue({
                ...formValue,
                projectId: event.currentTarget.value,
              })
            }
            value={formValue.projectId}
          >
            {projects.map(({ id, name }) => (
              <option key={id} value={id}>
                {name}
              </option>
            ))}
          </NativeSelect.Field>
          <NativeSelect.Indicator />
        </NativeSelect.Root>
      </Field.Root>

      {/* instance type */}
      <RadioCard.Root
        disabled={loading}
        onValueChange={(event) =>
          handleInstanceTypeChange(
            choices.find((choice) => choice.name === event.value)!,
          )
        }
        value={instanceType.name}
      >
        <RadioCard.Label>Type d'instance</RadioCard.Label>
        <HStack alignItems="stretch">
          {choices.map(({ cpuCores, diskBytes, memoryBytes, name }) => (
            <RadioCard.Item key={name} value={name}>
              <RadioCard.ItemHiddenInput />
              <RadioCard.ItemControl>
                <RadioCard.ItemContent>
                  <RadioCard.ItemText>Instance {name}</RadioCard.ItemText>
                  <RadioCard.ItemDescription>
                    {cpuCores} CPU, {bytesToGB(memoryBytes)} de RAM,{' '}
                    {bytesToGB(diskBytes)} de disque
                  </RadioCard.ItemDescription>
                </RadioCard.ItemContent>
                <RadioCard.ItemIndicator />
              </RadioCard.ItemControl>
            </RadioCard.Item>
          ))}
        </HStack>
      </RadioCard.Root>

      {/* snippet */}
      <Field.Root disabled={loading}>
        <HStack>
          <Field.Label>
            Snippet
            <Field.RequiredIndicator />
          </Field.Label>

          <Switch.Root
            onChange={() => setSnippetEditable(!isSnippetEditable)}
            checked={isSnippetEditable}
          >
            <Switch.HiddenInput />
            <Switch.Label>Editable</Switch.Label>
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
          </Switch.Root>
        </HStack>

        <AppCodeEditor
          editable={isSnippetEditable}
          code={formValue.snippet}
          lang="yaml"
          onChange={(snippet) => setFormValue({ ...formValue, snippet })}
        />
      </Field.Root>
    </VStack>
  );
};

const foo = `users:
  - name: \${OUR_VM_USER}
    gecos: "\${OUR_VM_USER}"
    shell: /bin/bash
    ssh-authorized-keys:
      - \${OUR_SSH_PUBLIC_KEY}
    sudo: ALL=(ALL) NOPASSWD:ALL

  - name: \${THEIR_VM_USER}
    gecos: "\${THEIR_VM_USER}"
    shell: /bin/bash
    ssh-authorized-keys:
      - \${THEIR_SSH_PUBLIC_KEY}
    sudo: ALL=(ALL) NOPASSWD:ALL

runcmd:
  - apt-get update
  - apt-get install -y ca-certificates curl gnupg lsb-release qemu-guest-agent
  
  # Configuration agent Qemu
  - systemctl enable qemu-guest-agent
  - systemctl start qemu-guest-agent

  # Configuration de fstrim pour rendre du stockage au Proxmox
  - sudo systemctl enable fstrim.timer
  - systemctl start fstrim.timer

  # Upgrade de l'image au démarrage de la VM, mais sans interaction utilisateur
  - DEBIAN_FRONTEND=noninteractive apt-get upgrade -y -o Dpkg::Options::="--force-confdef" -o Dpkg::Options::="--force-confold"`;
