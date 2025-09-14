import { Flex, Icon, Menu, Portal, Text } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { LuChevronsUpDown } from 'react-icons/lu';

import { setActiveOrganization, setActiveProject } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';

/**
 * Global project and organization switcher component.
 *
 * Displays dropdown menus for switching between organizations and projects.
 * Automatically filters projects based on the selected organization and updates
 * the application state when selections change.
 *
 * @returns The rendered project switcher component
 */
export const ProjectGlobalSwitcher: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization!,
  );
  const activeProject = useAppSelector(
    (state) => state.application.activeProject!,
  );
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );

  const organizationProjects = useAppSelector((state) =>
    state.resources.projects.filter(
      (project) =>
        project.organizationId === state.application.activeOrganization?.id,
    ),
  );

  return (
    <Flex alignItems="center" flexDir={{ base: 'column', sm: 'row' }} gap={2}>
      <SwitcherMenu
        format={(organization) => organization?.name || 'Aucune organisation'}
        onChange={(organization) =>
          dispatch(setActiveOrganization(organization))
        }
        options={organizations}
        value={activeOrganization}
      />
      <Text color="fg.subtle" display={{ base: 'none', sm: 'block' }}>
        /
      </Text>
      <SwitcherMenu
        format={(project) => project?.name || 'Aucun projet'}
        onChange={(project) => dispatch(setActiveProject(project))}
        options={organizationProjects}
        value={activeProject}
      />
    </Flex>
  );
};

/**
 * Props for the generic SwitcherMenu component.
 *
 * @template T - The type of options in the menu
 */
type ProjectSwitcherMenuProps<T> = {
  /** Function to format an option for display */
  format: (option: T) => string;
  /** Array of available options */
  options: T[];
  /** Callback fired when selection changes */
  onChange: (id: T) => void;
  /** Currently selected value */
  value: T;
};

/**
 * Generic dropdown menu component for switching between options.
 *
 * @template T - The type of options in the menu
 * @param props - Component props
 * @param props.format - Function to format an option for display
 * @param props.options - Array of available options
 * @param props.onChange - Callback fired when selection changes
 * @param props.value - Currently selected value
 * @returns The rendered switcher menu
 */
export const SwitcherMenu = <T,>({
  format,
  options,
  onChange,
  value,
}: ProjectSwitcherMenuProps<T>) => (
  <Menu.Root positioning={{ placement: 'bottom-start' }}>
    <Menu.Trigger
      borderWidth={1}
      alignItems="center"
      display="flex"
      gap="2"
      focusVisibleRing="outside"
      _focusVisible={{ bg: 'bg.muted' }}
      rounded="l2"
      p={1}
      height={8}
      maxWidth={{ base: 40, md: 'full' }}
    >
      <Text
        fontWeight="medium"
        fontSize="sm"
        ms="1"
        overflow="hidden"
        textOverflow="ellipsis"
        textWrap="nowrap"
      >
        {format(value)}
      </Text>
      <Icon color="fg.muted">
        <LuChevronsUpDown />
      </Icon>
    </Menu.Trigger>
    <Portal>
      <Menu.Positioner>
        <Menu.Content minW={64}>
          {options.map((option, index) => (
            <Menu.Item
              key={index}
              onSelect={() => onChange(option)}
              value={format(option)}
            >
              {format(option)}
            </Menu.Item>
          ))}
        </Menu.Content>
      </Menu.Positioner>
    </Portal>
  </Menu.Root>
);
