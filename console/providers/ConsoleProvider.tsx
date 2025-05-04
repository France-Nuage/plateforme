import {
  createInstance,
  fetchAllHypervisors,
  fetchAllInstances,
  registerHypervisor,
  setMode,
} from "@/features";
import { useAppDispatch, useAppSelector } from "@/hooks";
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
};

export const ConsoleProvider = forwardRef<Actions, Props>(
  ({ children }, ref) => {
    const dispatch = useAppDispatch();
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const application = useAppSelector((state) => state.application);

    useImperativeHandle(ref, () => ({
      changeMode: () => dispatch(setMode()),
      createInstance: (maxCpuCores, maxMemoryBytes, name) =>
        dispatch(createInstance({ maxCpuCores, maxMemoryBytes, name })),
      registerHypervisor: (authorizationToken, storageName, url) =>
        dispatch(registerHypervisor({ authorizationToken, storageName, url })),
    }));

    useEffect(() => {
useEffect(() => {
  dispatch(fetchAllHypervisors());
  dispatch(fetchAllInstances());
}, [application.mode, dispatch]);
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
    }, [application.mode, dispatch]);

    return (
      <DataProvider
        name="France Nuage"
        data={{ application, hypervisors, instances }}
      >
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = "France Nuage Console Provider";
