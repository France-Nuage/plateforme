import { PlasmicCanvasHost } from '@plasmicapp/loader-react';
import { registerComponent } from '@plasmicapp/react-web/lib/host';
import { FunctionComponent } from 'react';

import { ConsoleProvider } from '@/providers/ConsoleProvider';

export const PlasmicHost: FunctionComponent = () => <PlasmicCanvasHost />;

registerComponent(ConsoleProvider, {
  importPath: './src/providers/ConsoleProvider',
  name: 'ConsoleProvider',
  props: {
    children: 'slot',
  },
  providesData: true,
  refActions: {
    changeMode: {
      argTypes: [],
      description: 'Change the application mode',
    },
    createInstance: {
      argTypes: [
        { name: 'maxCpuCores', type: 'number' },
        { name: 'maxMemory', type: 'number' },
        { name: 'name', type: 'string' },
      ],
      description: 'Create a new instance',
    },
    registerHypervisor: {
      argTypes: [
        { name: 'authorizationToken', type: 'string' },
        { name: 'storageName', type: 'string' },
        { name: 'url', type: 'string' },
      ],
      description: 'Register a new hypervisor',
    },
    signin: {
      argTypes: [],
      description: 'Redirects the user to the authentication server.',
    },
    signout: {
      argTypes: [],
      description: 'Signs the user out.',
    },
  },
});
