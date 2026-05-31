import { Box, Button, Flex, Stack, Text } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { IconType } from 'react-icons';
import { useLocation } from 'react-router';
import { Link } from 'react-router';

import { useAppSelector } from '@/hooks';

export type SidebarLink = {
  Icon: IconType;
  label: string;
  to: string;
  /** When true, the link is only shown to platform admins. */
  adminOnly?: boolean;
};

export type SidebarSection = {
  title: string;
  links: SidebarLink[];
};

export type AppSidebarProps = {
  links: SidebarLink[];
};

export const AppSidebar: FunctionComponent<AppSidebarProps> = ({ links }) => {
  const location = useLocation();
  const isAdmin = useAppSelector((state) => state.authentication.isAdmin);

  const activeLink = links
    .filter(
      (l) =>
        location.pathname === l.to || location.pathname.startsWith(l.to + '/'),
    )
    .sort((a, b) => b.to.length - a.to.length)[0]?.to;

  const sections: SidebarSection[] = [
    {
      links: links.filter((link) => link.to.startsWith('/compute')),
      title: 'Compute',
    },
    {
      links: links.filter((link) => link.to.startsWith('/managed')),
      title: 'Services managés',
    },
    {
      // Admin links are a UX-only hint gated on the Keycloak 'admin' role.
      // The backend remains authoritative via users.is_admin.
      links: isAdmin ? links.filter((link) => link.adminOnly) : [],
      title: 'Administration',
    },
  ];

  return (
    <Box h="100%" w={320}>
      <Stack bg="bg.panel" p={{ base: 4, md: 6 }} gap={4}>
        {sections
          .filter((section) => section.links.length > 0)
          .map((section) => (
            <Flex direction="column" key={section.title}>
              <Text fontSize="sm" fontWeight="medium" color="fg.subtle">
                {section.title}
              </Text>
              {section.links.map(({ Icon, label, to }) => (
                <Button
                  aria-current={to === activeLink && 'page'}
                  bgColor={to === activeLink ? 'bg.subtle' : undefined}
                  gap={3}
                  justifyContent="start"
                  key={to}
                  variant="ghost"
                  width="full"
                  color="fg.muted"
                  _hover={{
                    bg: 'colorPalette.subtle',
                    color: 'colorPalette.fg',
                  }}
                  _currentPage={{ color: 'colorPalette.fg' }}
                  asChild
                >
                  <Link to={to}>
                    <Icon />
                    {label}
                  </Link>
                </Button>
              ))}
            </Flex>
          ))}
      </Stack>
    </Box>
  );
};
