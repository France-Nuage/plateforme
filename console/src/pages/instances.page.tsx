import { FunctionComponent, useMemo } from 'react';

import { InstanceTable } from '@/components';
import { useAppSelector } from '@/hooks';

/**
 * Instances page component.
 *
 * Displays a table of compute instances filtered by the currently active project.
 * The page retrieves all necessary data from the Redux store including instances,
 * zones, hypervisors, organizations, projects, and VPCs.
 *
 * @returns The rendered instances page with filtered instance table
 */
export const InstancesPage: FunctionComponent = () => {
  const zones = useAppSelector((state) => state.infrastructure.zones);
  const hypervisors = useAppSelector((state) => state.hypervisors.hypervisors);
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const projects = useAppSelector((state) => state.resources.projects);
  const vpcs = useAppSelector(
    (state) => state.infrastructure.zeroTrustNetworks,
  );
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const instances = useAppSelector((state) => state.instances.instances);
  const scopedInstances = useMemo(
    () =>
      instances.filter((instance) => instance.projectId === activeProject?.id),
    [activeProject, instances],
  );

  return (
    <InstanceTable
      zones={zones}
      hypervisors={hypervisors}
      instances={scopedInstances}
      organizations={organizations}
      projects={projects}
      vpcs={vpcs}
    />
  );
};
