export type ManagedService = {
  id: string;
  slug: string;
  name: string;
  description?: string;
  category: string;
  databaseEngine?: string;
  iconUrl?: string;
  createdAt: string;
};

export type ManagedServiceVersion = {
  id: string;
  serviceId: string;
  chartVersion: string;
  appVersion?: string;
  ociReference: string;
  configurableValuesSchema?: string;
  uiSchema?: string;
  createdAt: string;
};

export type ManagedServiceInstance = {
  id: string;
  serviceId: string;
  versionId: string;
  planId?: string;
  projectSlug: string;
  organizationSlug: string;
  namespace: string;
  releaseName: string;
  userValues?: string;
  status: ManagedInstanceStatus;
  createdAt: string;
};

export enum ManagedInstanceStatus {
  Provisioning = 'provisioning',
  Running = 'running',
  Upgrading = 'upgrading',
  Failed = 'failed',
  Deleting = 'deleting',
  Deleted = 'deleted',
}

export type ManagedServicePlanEntitlement = {
  key: string;
  label: string;
  value: string;
};

/**
 * Pricing model of a plan (FRA-15).
 *
 * - `flat`: fixed price, billed with quantity 1 (no seat selector).
 * - `per_unit`: fixed price per seat, billed with the chosen quantity.
 * - `tiered`: graduated/volume tiers priced per seat.
 *
 * `per_unit` and `tiered` require a seat quantity at checkout.
 */
export type PricingModel = 'flat' | 'per_unit' | 'tiered';

export type ManagedServicePlan = {
  id: string;
  serviceId: string;
  slug: string;
  name: string;
  description?: string;
  status: 'active' | 'archived';
  highlighted: boolean;
  valuesOverride?: string;
  entitlements: ManagedServicePlanEntitlement[];
  priceMonthlyCents?: number;
  priceYearlyCents?: number;
  requiresPayment: boolean;
  /**
   * Pricing model of the plan. Defaults to `flat` when the backend reports an
   * empty value, keeping pre-FRA-15 plans unchanged.
   */
  pricingModel: PricingModel;
  createdAt: string;
};

export type CreateManagedInstanceInput = {
  projectSlug: string;
  organizationSlug: string;
  serviceSlug: string;
  versionId: string;
  planId: string;
  userValues?: string;
  secretValues?: string;
};

export type UpgradeManagedInstanceInput = {
  instanceId: string;
  versionId: string;
  userValues?: string;
  secretValues?: string;
};

export type ConnectionInfoField = {
  key: string;
  label: string;
  value: string;
  display: 'text' | 'password' | 'copy';
  order: number;
};
