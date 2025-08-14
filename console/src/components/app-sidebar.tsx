import { Box, Button, Stack } from '@chakra-ui/react';
import { FunctionComponent } from 'react';
import { IconType } from 'react-icons';
import { useLocation } from 'react-router';
import { Link } from 'react-router';

export type AppSidebarProps = {
  links: { Icon: IconType; label: string; to: string }[];
};

export const AppSidebar: FunctionComponent<AppSidebarProps> = ({ links }) => {
  const location = useLocation();

  return (
    <Box h="100%" w={320}>
      <Stack bg="bg.panel" p={{ base: 4, md: 6 }}>
        {links.map(({ Icon, label, to }, index) => (
          <Button
            aria-current={location.pathname === to && 'page'}
            gap={3}
            justifyContent="start"
            key={to}
            variant="ghost"
            width="full"
            color="fg.muted"
            _hover={{ bg: 'colorPalette.subtle', color: 'colorPalette.fg' }}
            _currentPage={{ color: 'colorPalette.fg' }}
            asChild
          >
            <Link to={to}>
              <Icon />
              {label}
            </Link>
          </Button>
        ))}
      </Stack>
    </Box>
  );
};
