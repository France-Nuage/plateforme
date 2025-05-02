import {
  createInstance,
  fetchAllHypervisors,
  fetchAllInstances,
  registerHypervisor,
} from "@/features";
import { useAppDispatch, useAppSelector } from "@/hooks";
import { DataProvider } from "@plasmicapp/react-web/lib/host";
import { forwardRef, ReactNode, useEffect, useImperativeHandle } from "react";

export type Props = {
  children: ReactNode;
};

export type Actions = {
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

    useImperativeHandle(ref, () => ({
      createInstance: (maxCpuCores, maxMemoryBytes, name) =>
        dispatch(createInstance({ maxCpuCores, maxMemoryBytes, name })),
      registerHypervisor: (authorizationToken, storageName, url) =>
        dispatch(registerHypervisor({ authorizationToken, storageName, url })),
    }));

    useEffect(() => {
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
    }, [dispatch]);

    return (
      <DataProvider name="France Nuage" data={{ hypervisors, instances }}>
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = "France Nuage Console Provider";
