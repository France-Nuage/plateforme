import { ManagedInstanceStatus } from '@france-nuage/sdk';

/**
 * Visual identity of a managed instance status: badge color palette + label.
 *
 * Centralised so that table cells, detail pages and badges stay in sync.
 */
export const MANAGED_INSTANCE_STATUS_COLORS: Record<
  ManagedInstanceStatus,
  string
> = {
  [ManagedInstanceStatus.Provisioning]: 'blue',
  [ManagedInstanceStatus.Running]: 'green',
  [ManagedInstanceStatus.Upgrading]: 'orange',
  [ManagedInstanceStatus.Failed]: 'red',
  [ManagedInstanceStatus.Deleting]: 'orange',
  [ManagedInstanceStatus.Deleted]: 'gray',
};

export const MANAGED_INSTANCE_STATUS_LABELS: Record<
  ManagedInstanceStatus,
  string
> = {
  [ManagedInstanceStatus.Provisioning]: 'Provisionnement',
  [ManagedInstanceStatus.Running]: 'En service',
  [ManagedInstanceStatus.Upgrading]: 'Mise à niveau',
  [ManagedInstanceStatus.Failed]: 'En erreur',
  [ManagedInstanceStatus.Deleting]: 'Suppression en cours',
  [ManagedInstanceStatus.Deleted]: 'Supprimé',
};

/**
 * Catalog category visual identity: badge color palette + label.
 */
export const MANAGED_SERVICE_CATEGORY_LABELS: Record<string, string> = {
  analytics: 'Analytique',
  automation: 'Automatisation',
  cms: 'CMS',
  collaboration: 'Collaboration',
  dashboard: 'Dashboard',
  database: 'Base de données',
  erp: 'ERP',
  security: 'Sécurité',
  storage: 'Stockage',
};

export const MANAGED_SERVICE_CATEGORY_COLORS: Record<string, string> = {
  analytics: 'purple',
  automation: 'orange',
  cms: 'teal',
  collaboration: 'blue',
  dashboard: 'pink',
  database: 'green',
  erp: 'yellow',
  security: 'red',
  storage: 'cyan',
};

/**
 * Frequency at which a managed instance status is polled while the user
 * stays on a listing or detail page (provisioning / upgrading transitions
 * must be visible without manual refresh).
 */
export const MANAGED_INSTANCE_POLL_INTERVAL_MS = 5000;

/**
 * Safely parse a JSON string into a record. Returns null on any error or
 * when the input is empty/undefined.
 */
export function parseJsonObject(
  value: string | null | undefined,
): Record<string, unknown> | null {
  if (!value) return null;
  try {
    const parsed = JSON.parse(value) as unknown;
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      return parsed as Record<string, unknown>;
    }
    return null;
  } catch {
    return null;
  }
}
