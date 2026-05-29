import {
  Badge,
  Button,
  Flex,
  Grid,
  HStack,
  Heading,
  Input,
  Stack,
  Text,
} from '@chakra-ui/react';
import { FunctionComponent, useMemo } from 'react';
import { Link } from 'react-router';

import { ManagedServiceCard } from '@/components';
import { useAppSelector, useUrlState } from '@/hooks';
import {
  MANAGED_SERVICE_CATEGORY_COLORS,
  MANAGED_SERVICE_CATEGORY_LABELS,
} from '@/services';
import { Routes } from '@/types';

/**
 * Catalog of managed services available for deployment.
 *
 * The list of services is preloaded application-wide by
 * `useControlplaneData`; this page only takes care of presentation:
 * search and category filtering, both synced to URL search params.
 */
export const ManagedServicesPage: FunctionComponent = () => {
  const services = useAppSelector((state) => state.managedServices.services);
  const loading = useAppSelector((state) => state.managedServices.loading);
  const [search, setSearch] = useUrlState('search', '');
  const [activeCategory, setActiveCategory] = useUrlState('category', '');

  const categories = useMemo(
    () => [...new Set(services.map((service) => service.category))],
    [services],
  );

  const filteredServices = useMemo(() => {
    const needle = search.toLowerCase();
    return services.filter((service) => {
      const matchesSearch =
        service.name.toLowerCase().includes(needle) ||
        service.slug.toLowerCase().includes(needle) ||
        (service.description ?? '').toLowerCase().includes(needle);
      const matchesCategory =
        !activeCategory || service.category === activeCategory;
      return matchesSearch && matchesCategory;
    });
  }, [services, search, activeCategory]);

  return (
    <Stack gap={6}>
      <Flex justify="space-between" align="center" wrap="wrap" gap={4}>
        <Heading>Services managés</Heading>
        <Button variant="outline" asChild>
          <Link to={Routes.ManagedInstances}>Mes instances</Link>
        </Button>
      </Flex>

      <Flex gap={4} direction={{ base: 'column', md: 'row' }}>
        <Input
          placeholder="Rechercher un service..."
          value={search}
          onChange={(event) => setSearch(event.target.value)}
          maxW={{ md: '320px' }}
        />
        <HStack gap={2} flexWrap="wrap">
          <Badge
            cursor="pointer"
            variant={activeCategory === '' ? 'solid' : 'outline'}
            onClick={() => setActiveCategory('')}
          >
            Tous
          </Badge>
          {categories.map((category) => (
            <Badge
              key={category}
              cursor="pointer"
              colorPalette={MANAGED_SERVICE_CATEGORY_COLORS[category] ?? 'gray'}
              variant={activeCategory === category ? 'solid' : 'outline'}
              onClick={() =>
                setActiveCategory(activeCategory === category ? '' : category)
              }
            >
              {MANAGED_SERVICE_CATEGORY_LABELS[category] ?? category}
            </Badge>
          ))}
        </HStack>
      </Flex>

      {loading ? (
        <Text color="fg.muted">Chargement...</Text>
      ) : filteredServices.length === 0 ? (
        <Text color="fg.muted">Aucun service trouvé.</Text>
      ) : (
        <Grid
          templateColumns={{
            base: '1fr',
            lg: 'repeat(3, 1fr)',
            md: 'repeat(2, 1fr)',
          }}
          gap={4}
        >
          {filteredServices.map((service) => (
            <ManagedServiceCard key={service.id} service={service} />
          ))}
        </Grid>
      )}
    </Stack>
  );
};
