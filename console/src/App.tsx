import { useEffect, useState } from "react";
import {
  AppButton,
  AppDrawer,
  HypervisorInput,
  HypervisorTable,
  InstanceTable,
} from "./components";
import { useHypervisorService, useInstanceService } from "./hooks";
import { Hypervisor, Instance } from "./types";

const App = () => {
  const [draftHypervisor, setDraftHypervisor] = useState<
    Omit<Hypervisor, "id">
  >({ authorizationToken: "", storageName: "", url: "" });
  const [hypervisors, setHypervisors] = useState<Hypervisor[]>([]);
  const [instances, setInstances] = useState<Instance[]>([]);
  const hypervisorService = useHypervisorService();
  const instanceService = useInstanceService();
  const [isDrawerOpen, setDrawerOpen] = useState(false);

  useEffect(() => {
    hypervisorService?.list().then(setHypervisors);
  }, [hypervisorService]);

  useEffect(() => {
    instanceService?.list().then(setInstances);
  }, [instanceService]);

  return (
    <>
      <AppDrawer
        onClose={() => setDrawerOpen(false)}
        open={isDrawerOpen}
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
      <div className="container mx-auto space-y-4">
        <h1 className="text-3xl">FranceNuage</h1>
        <HypervisorTable
          hypervisors={hypervisors}
          onClick={() => setDrawerOpen(true)}
        />
        <InstanceTable instances={instances} onClick={() => {}} />
      </div>
    </>
  );
};

export default App;
