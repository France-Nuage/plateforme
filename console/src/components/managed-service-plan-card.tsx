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

/**
 * Displays a single managed-service plan as a selectable card.
 *
 * The price section adapts to the plan's `pricingModel` (FRA-15):
 * - `flat`: the monthly/yearly amount, billed with quantity 1.
 * - `per_unit`: the per-seat amount (`… / siège`) plus an indicative total
 *   (`per-seat × seats`) once a seat quantity is known.
 * - `tiered`: a "tarif dégressif" label — the exact amount is computed by
 *   Stripe at checkout, since the declining tiers are not projected onto the
 *   plan's two price columns.
 *
 * @param seats - The seat quantity chosen on the detail page. Used to render the
 *   indicative total for `per_unit` plans; ignored for `flat`/`tiered`.
 */
export const ManagedServicePlanCard: FunctionComponent<{
  plan: ManagedServicePlan;
  billingPeriod: 'monthly' | 'yearly';
  selected: boolean;
  onSelect: () => void;
  seats?: number;
}> = ({ plan, billingPeriod, selected, onSelect, seats }) => {
  const intervalLabel = billingPeriod === 'monthly' ? 'mois' : 'an';
  const isPerSeat =
    plan.pricingModel === 'per_unit' || plan.pricingModel === 'tiered';

  const unitCents = useMemo(
    () =>
      billingPeriod === 'monthly'
        ? plan.priceMonthlyCents
        : plan.priceYearlyCents,
    [plan.priceMonthlyCents, plan.priceYearlyCents, billingPeriod],
  );

  // Indicative total for per_unit plans: per-seat amount × seats. Tiered plans
  // carry no per-seat amount here, so no total is computed client-side.
  const totalCents = useMemo(() => {
    if (plan.pricingModel !== 'per_unit') return null;
    if (unitCents === undefined || !seats || seats < 1) return null;
    return unitCents * seats;
  }, [plan.pricingModel, unitCents, seats]);

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
          {plan.pricingModel === 'tiered' ? (
            <Text fontSize="lg" fontWeight="bold">
              Tarif dégressif
              <Text
                as="span"
                fontSize="sm"
                fontWeight="normal"
                color="fg.muted"
              >
                {' '}
                / siège / {intervalLabel}
              </Text>
            </Text>
          ) : (
            unitCents !== undefined && (
              <Stack gap={0}>
                <Flex align="baseline" gap={1}>
                  <Text fontSize="3xl" fontWeight="bold">
                    {formatPrice(unitCents)}
                  </Text>
                  <Text fontSize="sm" color="fg.muted">
                    EUR{' '}
                    {isPerSeat
                      ? `/ siège / ${intervalLabel}`
                      : `/${intervalLabel}`}
                  </Text>
                </Flex>
                {totalCents !== null && (
                  <Text fontSize="sm" color="fg.muted">
                    Soit {formatPrice(totalCents)} EUR /{intervalLabel} pour{' '}
                    {seats} siège{seats && seats > 1 ? 's' : ''}
                  </Text>
                )}
              </Stack>
            )
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
