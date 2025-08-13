import { Box, Drawer, Flex, Stack } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Outlet } from 'react-router';

import { AppHeader, AppSidebar, AppSidebarProps } from '@/components';

export type AppLayoutProps = {
  links: AppSidebarProps['links'];
};

export const AppLayout: FunctionComponent<AppLayoutProps> = ({ links }) => {
  return (
    <Box colorPalette={'blue'}>
      <Drawer.Root placement="start">
        <Stack h="100vh" gap={0}>
          <AppHeader />
          <Flex flex={1} overflow="hidden">
            <Box
              borderRightWidth={1}
              display={{ base: 'none', lg: 'block' }}
              h="100%"
            >
              <AppSidebar links={links} />
            </Box>
            <Box flex={1} p={4} overflowY="auto">
              <Outlet />
            </Box>
          </Flex>
        </Stack>
        <Drawer.Backdrop />
        <Drawer.Positioner>
          <Drawer.Content>
            <AppSidebar links={links} />
          </Drawer.Content>
        </Drawer.Positioner>
      </Drawer.Root>
    </Box>
  );
};
