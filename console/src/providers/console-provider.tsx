import { forwardRef, ReactNode, useEffect, useImperativeHandle } from "react";
import { DataProvider } from "@plasmicapp/loader-react";
import { useAppDispatch, useAppSelector } from "@/hooks";
import {
  addHypervisor,
  fetchAllHypervisors,
} from "@/features/hypervisors.slice";
import { hypervisor } from "@/fixtures";
import { createInstance, fetchAllInstances } from "@/features/instances.slice";

export type Props = {
  children: ReactNode;
};

export type Actions = {
  registerHypervisor: () => void;
  createInstance: (
    maxCpuCores: number,
    maxMemoryBytes: number,
    name: string,
  ) => void;
};

export const ConsoleProvider = forwardRef<Actions, Props>(
  ({ children }, ref) => {
    const hypervisors = useAppSelector(
      (state) => state.hypervisors.hypervisors,
    );
    const instances = useAppSelector((state) => state.instances.instances);
    const dispatch = useAppDispatch();

    useEffect(() => {
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
    }, []);

    useImperativeHandle(ref, () => ({
      registerHypervisor: () => dispatch(addHypervisor(hypervisor())),
      createInstance: (maxCpuCores, maxMemoryBytes, name) =>
        dispatch(createInstance({ maxCpuCores, maxMemoryBytes, name })),
    }));

    return (
      <DataProvider name="France Nuage" data={{ hypervisors, instances }}>
        {children}
      </DataProvider>
    );
  },
);

ConsoleProvider.displayName = "Console Provider";
