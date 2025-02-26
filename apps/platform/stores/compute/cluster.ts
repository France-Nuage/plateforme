import type { Cluster } from "@france-nuage/types";

interface State {
  clusters: Cluster[];
}

export const useClusterStore = defineStore("cluster", {
  state: (): State => ({
    clusters: [],
  }),
  actions: {
    loadClusters: async function (): Promise<void> {
      const { $api } = useNuxtApp();

      return $api().compute.clusters.list().then();
    },
  },
});
