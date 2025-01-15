interface State {
  regions: Array<any>;
  region: any;
}

export const useRegionStore = defineStore("region", {
  state: (): State => ({
    regions: [],
    region: null,
  }),
  actions: {
    loadRegions: async function (queryParams?: any): Promise<void> {
      const { $api } = useNuxtApp();

      return $api()
        .compute.regions.list(queryParams)
        .then((response) => {
          this.regions = response.data;
          return response;
        });
    },
    loadRegion: async function (id: string, queryParams?: any): Promise<void> {
      const { $api } = useNuxtApp();

      $api()
        .compute.regions.get(id)
        .then(({ data }) => {
          this.region = data;
        });
    },
    createRegion: async function (data) {
      const { $api } = useNuxtApp();

      $api()
        .compute.regions.post(data)
        .then(({ data }) => {
          this.region = data;
        });
    },
  },
});
