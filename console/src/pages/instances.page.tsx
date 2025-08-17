import { FunctionComponent } from 'react';

import { InstanceTable } from '@/components';
import { useAppSelector } from '@/hooks';

export const InstancesPage: FunctionComponent = () => {
  const instances = useAppSelector((state) => state.instances.instances);
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
