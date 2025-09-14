import { FunctionComponent } from 'react';

import { InstanceTable } from '@/components';
import { useAppSelector } from '@/hooks';

/**
 * Instances page component.
 *
 * Displays a table of compute instances filtered by the currently active project.
 * The page retrieves all necessary data from the Redux store including instances,
 * datacenters, hypervisors, organizations, projects, and VPCs.
 *
 * @returns The rendered instances page with filtered instance table
 */
export const InstancesPage: FunctionComponent = () => {
  const datacenters = useAppSelector(
    (state) => state.infrastructure.datacenters,
  );
  const hypervisors = useAppSelector((state) => state.hypervisors.hypervisors);
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const projects = useAppSelector((state) => state.resources.projects);
  const vpcs = useAppSelector(
    (state) => state.infrastructure.zeroTrustNetworks,
  );

  // Filter instances to show only those belonging to the active project
  const instances = useAppSelector((state) =>
    state.instances.instances.filter(
      (instance) => instance.projectId === state.application.activeProject?.id,
    ),
  );

  return (
    <InstanceTable
      datacenters={datacenters}
      hypervisors={hypervisors}
      instances={instances}
      organizations={organizations}
      projects={projects}
      vpcs={vpcs}
    />
  );
};
