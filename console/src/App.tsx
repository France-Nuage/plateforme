import { useEffect, useState } from "react";
import {
  AppButton,
  AppDrawer,
  HypervisorInput,
  HypervisorTable,
  InstanceInput,
  InstanceTable,
} from "./components";
import { useHypervisorService, useInstanceService } from "./hooks";
import { Hypervisor, HypervisorFormValue, Instance, InstanceFormValue } from "./types";

const App = () => {
  const [draftHypervisor, setDraftHypervisor] = useState<
    HypervisorFormValue>({
      authorizationToken: "PVEAPIToken=",
      storageName: "local-lvm",
      url: "https://pvedev-dc03-internal.france-nuage.fr",
    });
  const [draftInstance, setDraftInstance] = useState<InstanceFormValue>({ name: '', maxCpuCores: 0, maxMemoryBytes: 0 });
  const [hypervisors, setHypervisors] = useState<Hypervisor[]>([]);
  const [instances, setInstances] = useState<Instance[]>([]);
  const hypervisorService = useHypervisorService();
  const instanceService = useInstanceService();
  const [isCreateHypervisorDrawerOpen, setCreateHypervisorDrawerOpen] = useState(false);
  const [isCreateInstanceDrawerOpen, setCreateInstanceDrawerOpen] = useState(false);

  useEffect(() => {
    hypervisorService?.list().then(setHypervisors);
  }, [hypervisorService]);

  useEffect(() => {
    instanceService?.list().then(setInstances);
  }, [instanceService]);

  return (
    <>
      <AppDrawer
        onClose={() => setCreateHypervisorDrawerOpen(false)}
        open={isCreateHypervisorDrawerOpen}
        title="Ajouter un hyperviseur"
      >
        <HypervisorInput
          onChange={setDraftHypervisor}
          value={draftHypervisor}
        />
        <AppButton
          className="mt-4"
          onClick={() => hypervisorService!.create(draftHypervisor)}
        >
          Ajouter l&apos;hyperviseur
        </AppButton>
      </AppDrawer>
      <AppDrawer onClose={() => setCreateInstanceDrawerOpen(false)} open={isCreateInstanceDrawerOpen} title="Ajouter une instance">
        <InstanceInput
          onChange={setDraftInstance}
          value={draftInstance}
        />
        <AppButton
          className="mt-4" onClick={() => { }}>Ajouter l&apos;instance</AppButton>
      </AppDrawer>
      <div className="container mx-auto space-y-4">
        <h1 className="text-3xl">FranceNuage</h1>
        <HypervisorTable
          hypervisors={hypervisors}
          onClick={() => setCreateHypervisorDrawerOpen(true)}
        />
        <InstanceTable instances={instances} onClick={() => setCreateInstanceDrawerOpen(true)} />
      </div>
    </>
  );
};

export default App;
