import * as React from 'react';
import { PlasmicCanvasHost, registerComponent } from '@plasmicapp/react-web/lib/host';
import { ConsoleProvider } from '@/providers/ConsoleProvider';

// You can register any code components that you want to use here; see
// https://docs.plasmic.app/learn/code-components-ref/
// And configure your Plasmic project to use the host url pointing at
// the /plasmic-host page of your nextjs app (for example,
// http://localhost:3000/plasmic-host).  See
// https://docs.plasmic.app/learn/app-hosting/#set-a-plasmic-project-to-use-your-app-host

// registerComponent(...)

export default function PlasmicHost() {
  return <PlasmicCanvasHost />;
}

registerComponent(ConsoleProvider, {
  name: "ConsoleProvider",
  props: {
    children: "slot",
  },
  providesData: true,
  refActions: {
    regiterHypervisor: {
      description: "Register a new hypervisor",
      argTypes: [
        { name: "authorizationToken", type: "string" },
        { name: "storageName", type: "string" },
        { name: "url", type: "string" },
      ]
    },
    createInstance: {
      description: "Create a new instance",
      argTypes: [
        { name: "maxCpuCores", type: "number" },
        { name: "maxMemory", type: "number" },
        { name: "name", type: "string" },
      ]
    }
  },
  importPath: './providers/ConsoleProvider'
})
