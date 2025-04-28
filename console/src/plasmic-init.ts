import { initPlasmicLoader } from "@plasmicapp/loader-react";
import { ConsoleProvider } from "./providers";

export const PLASMIC = initPlasmicLoader({
  projects: [
    {
      id: import.meta.env.VITE_PLASMIC_ID,
      token: import.meta.env.VITE_PLASMIC_TOKEN,
    },
  ],
  preview: true,
});

PLASMIC.registerComponent(ConsoleProvider, {
  name: "Console Provider",
  props: {
    children: "slot",
  },
  providesData: true,
  refActions: {
    registerHypervisor: {
      description: "Register a new hypervisor",
      argTypes: [],
    },
    createInstance: {
      description: "Create a new instance",
      argTypes: [
        { name: "maxCpuCores", type: "number" },
        { name: "maxMemory", type: "number" },
        { name: "name", type: "string" },
      ],
    },
  },
});
