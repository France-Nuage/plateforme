import {
  Badge,
  Box,
  Card,
  Flex,
  HStack,
  Heading,
  Stack,
  Text,
} from '@chakra-ui/react';
import { ManagedServicePlan } from '@france-nuage/sdk';
import { FunctionComponent, useMemo, useState } from 'react';

function formatPrice(cents: number): string {
  return (cents / 100).toFixed(2).replace('.', ',');
}

export const ManagedServicePlanCard: FunctionComponent<{
  plan: ManagedServicePlan;
  selected: boolean;
  onSelect: () => void;
}> = ({ plan, selected, onSelect }) => {
  const [billingPeriod, setBillingPeriod] = useState<'monthly' | 'yearly'>(
    'monthly',
  );

  const displayPrice = useMemo(() => {
    const cents =
      billingPeriod === 'monthly'
        ? plan.priceMonthlyCents
        : plan.priceYearlyCents;
    if (cents === undefined) return null;
    return formatPrice(cents);
  }, [plan.priceMonthlyCents, plan.priceYearlyCents, billingPeriod]);

  return (
    <Card.Root
      cursor="pointer"
      onClick={onSelect}
      borderWidth="2px"
      borderColor={selected ? 'blue.500' : 'border'}
      _hover={{ borderColor: selected ? 'blue.500' : 'blue.200' }}
      transition="border-color 0.15s"
    >
      <Card.Body>
        <Stack gap={3}>
          <Flex justify="space-between" align="start">
            <Heading size="md">{plan.name}</Heading>
            {plan.highlighted && (
              <Badge colorPalette="blue" variant="solid" size="sm">
                Recommande
              </Badge>
            )}
          </Flex>

          {plan.description && (
            <Text fontSize="sm" color="fg.muted">
              {plan.description}
            </Text>
          )}

          {plan.entitlements.length > 0 && (
            <HStack gap={2} flexWrap="wrap">
              {plan.entitlements.map((entitlement) => (
                <Badge
                  key={entitlement.key}
                  variant="outline"
                  size="sm"
                  title={entitlement.label}
                >
                  {entitlement.label}: {entitlement.value}
                </Badge>
              ))}
            </HStack>
          )}

          {displayPrice !== null && (
            <Box>
              <Text fontSize="2xl" fontWeight="bold">
                {displayPrice} EUR
              </Text>
              <HStack gap={2} mt={1}>
                <Text
                  fontSize="xs"
                  cursor="pointer"
                  fontWeight={billingPeriod === 'monthly' ? 'bold' : 'normal'}
                  color={
                    billingPeriod === 'monthly' ? 'fg' : 'fg.muted'
                  }
                  onClick={(e) => {
                    e.stopPropagation();
                    setBillingPeriod('monthly');
                  }}
                >
                  /mois
                </Text>
                <Text fontSize="xs" color="fg.muted">
                  |
                </Text>
                <Text
                  fontSize="xs"
                  cursor="pointer"
                  fontWeight={billingPeriod === 'yearly' ? 'bold' : 'normal'}
                  color={
                    billingPeriod === 'yearly' ? 'fg' : 'fg.muted'
                  }
                  onClick={(e) => {
                    e.stopPropagation();
                    setBillingPeriod('yearly');
                  }}
                >
                  /an
                </Text>
              </HStack>
            </Box>
          )}
        </Stack>
      </Card.Body>
    </Card.Root>
  );
};
