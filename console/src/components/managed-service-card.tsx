import { Badge, Box, Card, Stack, Text } from '@chakra-ui/react';
import { ManagedService } from '@france-nuage/sdk';
import { FunctionComponent } from 'react';
import { Link } from 'react-router';

import {
  MANAGED_SERVICE_CATEGORY_COLORS,
  MANAGED_SERVICE_CATEGORY_LABELS,
} from '@/services';

/**
 * Catalog card for a managed service. The whole card is a link to the
 * service detail page.
 */
export const ManagedServiceCard: FunctionComponent<{
  service: ManagedService;
}> = ({ service }) => {
  return (
    <Box asChild>
      <Link
        to={`/managed-services/${service.slug}`}
        style={{ textDecoration: 'none' }}
      >
        <Card.Root
          _hover={{ borderColor: 'colorPalette.fg', shadow: 'md' }}
          transition="all 0.2s"
          cursor="pointer"
          h="100%"
        >
          <Card.Header>
            <Stack gap={1}>
              <Card.Title>{service.name}</Card.Title>
              <Badge
                size="sm"
                colorPalette={
                  MANAGED_SERVICE_CATEGORY_COLORS[service.category] ?? 'gray'
                }
                variant="subtle"
                w="fit-content"
              >
                {MANAGED_SERVICE_CATEGORY_LABELS[service.category] ??
                  service.category}
              </Badge>
            </Stack>
          </Card.Header>
          <Card.Body>
            <Text fontSize="sm" color="fg.muted" lineClamp={3}>
              {service.description ?? 'Aucune description disponible.'}
            </Text>
          </Card.Body>
        </Card.Root>
      </Link>
    </Box>
  );
};
