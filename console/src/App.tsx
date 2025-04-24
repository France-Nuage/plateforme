import { useEffect, useState } from "react";
import { AppButton, AppDrawer, HypervisorInput } from "./components";
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
  const [isDrawerOpen, setDrawerOpen] = useState(true);

  useEffect(() => {
    hypervisorService?.list().then(setHypervisors);
  }, [hypervisorService]);

  useEffect(() => {
    instanceService?.list().then(setInstances)
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
        <AppButton className="mt-4" onClick={() => hypervisorService!.create(draftHypervisor)}>
          Ajouter l&apos;hyperviseur
        </AppButton>
      </AppDrawer>
      <div className="container mx-auto">
        <h1 className="text-3xl">FranceNuage</h1>

        <h2 className="text-xl">Hyperviseurs</h2>
        <AppButton onClick={() => setDrawerOpen(true)}>
          Ajouter un hyperviseur
        </AppButton>
        {hypervisors.length > 0 ? (
          <ul>
            {hypervisors.map((hypervisor) => (
              <li key={hypervisor.id}>hypervisor - {hypervisor.url}</li>
            ))}
          </ul>
        ) : (
          <div>aucun hyperviseur trouvé</div>
        )}

        <h2 className="text-xl">Instances</h2>
        <AppButton>Ajouter une instance</AppButton>
        {instances.length > 0 ? (
          <ul>
            {instances.map((instance) => (
              <li key={instance.id}>instance - {instance.name}</li>
            ))}
          </ul>
        ) : (
          <div>aucune instance trouvée</div>
        )}
      </div>
    </>
  );
};

export default App;
