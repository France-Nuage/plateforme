import { initPlasmicLoader } from "@plasmicapp/loader-react";
import { ConsoleProvider } from "./providers";
import { AppButton } from "./components";

export const PLASMIC = initPlasmicLoader({
  projects: [
    {
      id: "8hZRXuNQRs8zUDmvMoCc7h",
      token:
        "QQ0Mlvm9YpK8VqsEz3d6wuNKTm6L7Kr0exoGgGAwgtUG18TNWfJPKHpbzSG2XL1nLlIEDBqG6r886pJqg",
    },
  ],
  preview: true,
});

PLASMIC.registerComponent(AppButton, {
  name: "App Button",
  props: {},
});

PLASMIC.registerComponent(ConsoleProvider, {
  name: "Hypervisors Provider",
  props: {
    children: "slot",
  },
  providesData: true,
  refActions: {
    add: {
      description: "Add a new hypervisor",
      argTypes: [],
    },
  },
});
