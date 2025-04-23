import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { useEffect, useState } from "react";
import {
  Hypervisor,
  HypervisorsClient,
  InstanceInfo,
  InstancesClient,
} from "@/protocol";
import { AppButton, AppDrawer, HypervisorInput } from "./components";

export const transport = new GrpcWebFetchTransport({
  baseUrl: import.meta.env.VITE_CONTROLPLANE_URL,
  format: "binary",
});

const instancesClient = new InstancesClient(transport);
const hypervisorsClient = new HypervisorsClient(transport);

const App = () => {
  const [draftHypervisor, setDraftHypervisor] = useState<Hypervisor>({ url: '' });
  const [hypervisors, setHypervisors] = useState<Hypervisor[]>([]);
  const [instances, setInstances] = useState<InstanceInfo[]>([]);
  const [isDrawerOpen, setDrawerOpen] = useState(true);

  useEffect(() => {
    hypervisorsClient
      .listHypervisors({})
      .response.then(({ hypervisors }) => setHypervisors(hypervisors));
    instancesClient
      .listInstances({})
      .response.then(({ instances }) => setInstances(instances));
  }, []);

  return (
    <>
      <AppDrawer
        onClose={() => setDrawerOpen(false)}
        open={isDrawerOpen}
        title="Ajouter un hyperviseur"
      >
        <HypervisorInput onChange={setDraftHypervisor} value={draftHypervisor} />
        <AppButton>Ajouter l'hyperviseur</AppButton>
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
              <li>hypervisor - {hypervisor.url}</li>
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
              <li>instance - {instance.name}</li>
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
