import { Flex, Icon, Menu, Portal, Text } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { LuChevronsUpDown } from 'react-icons/lu';

import { setActiveOrganization, setActiveProject } from '@/features';
import { useAppDispatch, useAppSelector } from '@/hooks';

export const ProjectGlobalSwitcher: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization,
  );
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const projects = useAppSelector((state) => state.resources.projects);

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
        options={projects}
        value={activeProject}
      />
    </Flex>
  );
};

type ProjectSwitcherMenuProps<T> = {
  format: (option: T) => string;
  options: T[];
  onChange: (id: T) => void;
  value: T;
};

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
          <Menu.ItemGroup>foobar</Menu.ItemGroup>
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
