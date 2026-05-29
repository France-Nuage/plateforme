import { Badge } from '@chakra-ui/react';
import { ManagedInstanceStatus } from '@france-nuage/sdk';
import { FunctionComponent } from 'react';

import {
  MANAGED_INSTANCE_STATUS_COLORS,
  MANAGED_INSTANCE_STATUS_LABELS,
} from '@/services';

/**
 * Badge displaying a managed instance status with its localized label
 * and color palette. Centralises styling decisions previously duplicated
 * across listing and detail pages.
 */
export const ManagedInstanceStatusBadge: FunctionComponent<{
  status: ManagedInstanceStatus;
}> = ({ status }) => (
  <Badge colorPalette={MANAGED_INSTANCE_STATUS_COLORS[status]} variant="subtle">
    {MANAGED_INSTANCE_STATUS_LABELS[status] ?? status}
  </Badge>
);
