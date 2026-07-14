import { IconButton } from '@chakra-ui/react';
import { ManagedServiceInstance } from '@france-nuage/sdk';
import { FunctionComponent } from 'react';
import { HiInformationCircle } from 'react-icons/hi';
import { Link } from 'react-router';

export const DetailInstanceButton: FunctionComponent<{
  instance: ManagedServiceInstance;
}> = ({ instance }) => (
  <IconButton aria-label="voir le detail" asChild>
    <Link to={`/managed-services/instances/${instance.id}`}>
      <HiInformationCircle />
    </Link>
  </IconButton>
);
