import {
  Alert,
  Box,
  Link as ChakraLink,
  Drawer,
  Flex,
  Stack,
} from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { Link, Outlet } from 'react-router';

import { AppHeader, AppSidebar, AppSidebarProps } from '@/components';

/**
 * Props for the AppLayout component.
 */
export type AppLayoutProps = {
  /** Navigation links to display in the sidebar */
  links: AppSidebarProps['links'];
};

/**
 * Main application layout component.
 *
 * Provides the core application structure including header, sidebar navigation,
 * and main content area. Features responsive design with drawer-based navigation
 * on mobile devices and fixed sidebar on larger screens.
 *
 * @param props - Component props
 * @param props.links - Navigation links to display in the sidebar
 * @returns The rendered application layout
 */
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
            <Stack flex={1} p={4} overflowY="auto">
              <Stack>
                <Alert.Root mb={4}>
                  <Alert.Indicator />
                  <Alert.Content>
                    <Alert.Title>
                      Besoin d'une nouvelle application ou de davantage de
                      ressources ?
                      <ChakraLink asChild ml={2} variant="underline">
                        <Link to="mailto:support@france-nuage.fr">
                          Contactez-nous
                        </Link>
                      </ChakraLink>
                    </Alert.Title>
                  </Alert.Content>
                </Alert.Root>
              </Stack>
              <Outlet />
            </Stack>
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
