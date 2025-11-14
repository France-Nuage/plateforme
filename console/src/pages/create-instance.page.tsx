import { FunctionComponent } from 'react';

import { InstanceForm } from '@/components';
import { useAppSelector } from '@/hooks';

export const CreateInstancePage: FunctionComponent = () => {
  const projects = useAppSelector((state) => state.resources.projects);

  return projects.length ? <InstanceForm projects={projects} /> : <div />;
};
