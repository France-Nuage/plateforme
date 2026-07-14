import {
  Badge,
  Card,
  Flex,
  Heading,
  List,
  Stack,
  Text,
} from '@chakra-ui/react';
import { ManagedServicePlan } from '@france-nuage/sdk';
import { FunctionComponent, useMemo } from 'react';
import { LuCheck } from 'react-icons/lu';

function formatPrice(cents: number): string {
  return (cents / 100).toFixed(2).replace('.', ',');
}

export const ManagedServicePlanCard: FunctionComponent<{
  plan: ManagedServicePlan;
  billingPeriod: 'monthly' | 'yearly';
  selected: boolean;
  onSelect: () => void;
}> = ({ plan, billingPeriod, selected, onSelect }) => {
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
      <Card.Header>
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
      </Card.Header>

      <Card.Body>
        <Stack gap={4}>
          {displayPrice !== null && (
            <Flex align="baseline" gap={1}>
              <Text fontSize="3xl" fontWeight="bold">
                {displayPrice}
              </Text>
              <Text fontSize="sm" color="fg.muted">
                EUR /{billingPeriod === 'monthly' ? 'mois' : 'an'}
              </Text>
            </Flex>
          )}

          {plan.entitlements.length > 0 && (
            <List.Root gap={2} variant="plain">
              {plan.entitlements.map((entitlement) => (
                <List.Item key={entitlement.key} fontSize="sm">
                  <List.Indicator asChild color="green.500">
                    <LuCheck />
                  </List.Indicator>
                  {entitlement.label}: {entitlement.value}
                </List.Item>
              ))}
            </List.Root>
          )}
        </Stack>
      </Card.Body>
    </Card.Root>
  );
};
