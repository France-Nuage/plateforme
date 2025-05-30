import { PlasmicCanvasHost } from '@plasmicapp/loader-react';
import { registerComponent } from '@plasmicapp/react-web/lib/host';
import { FunctionComponent } from 'react';

import { ConsoleProvider } from '@/providers/ConsoleProvider';

export const PlasmicHost: FunctionComponent = () => <PlasmicCanvasHost />;

registerComponent(ConsoleProvider, {
  name: 'ConsoleProvider',
  props: {
    children: 'slot',
  },
  providesData: true,
  refActions: {
    createInstance: {
      description: 'Create a new instance',
      argTypes: [
        { name: 'maxCpuCores', type: 'number' },
        { name: 'maxMemory', type: 'number' },
        { name: 'name', type: 'string' },
      ],
    },
    changeMode: {
      description: 'Change the application mode',
      argTypes: [],
    },
    registerHypervisor: {
      description: 'Register a new hypervisor',
      argTypes: [
        { name: 'authorizationToken', type: 'string' },
        { name: 'storageName', type: 'string' },
        { name: 'url', type: 'string' },
      ],
    },
    signin: {
      description: 'Redirects the user to the authentication server.',
      argTypes: [],
    },
    signout: {
      description: 'Signs the user out.',
      argTypes: [],
    },
  },
  importPath: './src/providers/ConsoleProvider',
});
