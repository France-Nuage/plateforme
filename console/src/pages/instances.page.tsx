import { Button } from '@chakra-ui/react';
import { FunctionComponent } from 'react';

import { useAppSelector } from '@/hooks';

export const InstancesPage: FunctionComponent = () => {
  const instances = useAppSelector((state) => state.instances.instances);
  return (
    <>
      <Button>
        Besoin d'une nouvelle application ou de plus de ressources?
      </Button>
      <ul>
        {instances.map((instance) => (
          <li key={instance.id}>{instance.name}</li>
        ))}
      </ul>
    </>
  );
};
