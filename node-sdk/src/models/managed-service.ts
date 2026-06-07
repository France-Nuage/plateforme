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
  projectId: string;
  organizationId: string;
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
  createdAt: string;
};

export type CreateManagedInstanceInput = {
  projectId: string;
  organizationId: string;
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
