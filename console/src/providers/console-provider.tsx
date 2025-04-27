import { forwardRef, ReactNode, useEffect, useImperativeHandle, useState } from "react"
import { DataProvider } from "@plasmicapp/loader-react";
import { hypervisors as seed } from "@/fixtures"
import { Hypervisor, Instance } from "@/types";
import { useServices } from "@/hooks/use-services";

export type Props = {
  children: ReactNode,
}

export type Actions = {}

export const ConsoleProvider = forwardRef<Actions, Props>(({ children }, ref) => {
  const { hypervisor, instance } = useServices();
  const [hypervisors, setHypervisors] = useState<Hypervisor[]>([]);
  const [instances, setInstances] = useState<Instance[]>([]);

  useEffect(() => {
    hypervisor.list().then(setHypervisors);
  }, [hypervisor]);

  useEffect(() => {
    instance.list().then(setInstances);
  }, [instance]);

  useImperativeHandle(ref, () => ({
    add: () => setHypervisors([...hypervisors, ...seed(1)])
  }));

  return <DataProvider name="hypervisors" data={hypervisors}>{children}</DataProvider>
});
