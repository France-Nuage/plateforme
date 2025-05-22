import {
  clearAuthenticationState,
  createInstance,
  fetchAllHypervisors,
  fetchAllInstances,
  registerHypervisor,
  setMode,
} from "@/features";
import { useAppDispatch, useAppSelector } from "@/hooks";
import { userManager } from "@/services";
import { DataProvider } from "@plasmicapp/react-web/lib/host";
import { forwardRef, ReactNode, useEffect, useImperativeHandle } from "react";

export type Props = {
  children: ReactNode;
};

export type Actions = {
  changeMode: () => void;
  createInstance: (
    maxCpuCores: number,
    maxMemoryBytes: number,
    name: string,
  ) => void;
  registerHypervisor: (
    authorizationToken: string,
    storageName: string,
    url: string,
  ) => void;
  signin: () => void;
  signout: () => void;
};

export const ConsoleProvider = forwardRef<Actions, Props>(
  ({ children }, ref) => {
    const dispatch = useAppDispatch();

    const application = useAppSelector((state) => state.application);
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const user = useAppSelector((state) => state.authentication.user);

    useImperativeHandle(ref, () => ({
      changeMode: () => dispatch(setMode()),
      createInstance: (maxCpuCores, maxMemoryBytes, name) =>
        dispatch(createInstance({ maxCpuCores, maxMemoryBytes, name })),
      registerHypervisor: (authorizationToken, storageName, url) =>
        dispatch(registerHypervisor({ authorizationToken, storageName, url })),
      signin: () => userManager.signinRedirect(),
      signout: async () => {
        await userManager.removeUser();
        dispatch(clearAuthenticationState());
      },
    }));

    useEffect(() => {
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
    }, [application.mode, dispatch]);

    return (
      <DataProvider
        name="France Nuage"
        data={{ application, hypervisors, instances, user }}
      >
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = "France Nuage Console Provider";
