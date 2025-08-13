import {
  Box,
  Drawer,
  Flex,
  Heading,
  IconButton,
  Image,
} from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { HiMenu } from 'react-icons/hi';

import { ProjectGlobalSwitcher } from '@/components';

import { ColorModeButton } from './chakra';

export const AppHeader: FunctionComponent = () => (
  <Box
    as="header"
    borderBottomWidth={1}
    py={{ base: 2 }}
    px={{ base: 2, md: 4 }}
  >
    <Flex gap={{ base: 2, md: 4 }} alignItems="center">
      <Box display={{ lg: 'none' }}>
        <Drawer.Context>
          {(store) => (
            <IconButton
              colorPalette="gray"
              onClick={() => store.setOpen(!store.open)}
              variant="ghost"
            >
              <HiMenu />
            </IconButton>
          )}
        </Drawer.Context>
      </Box>
      <Image
        src="/logo.png"
        display={{ base: 'none', md: 'block' }}
        h={42}
        paddingY={1}
      />
      <Heading size="md" display={{ base: 'none', lg: 'block' }}>
        France Nuage
      </Heading>
      <ProjectGlobalSwitcher />
      <Box flexGrow={1} />
      <ColorModeButton colorPalette="gray" />
    </Flex>
  </Box>
);
