export const useZoneStore = defineStore("zone", {
  state: () => ({
    zones: [],
    zone: null,
    currentZone: null,
  }),
  actions: {
    loadZones: async function (): Promise<void> {
      const { $api } = useNuxtApp();

      $api()
        .compute.zones.list()
        .then(({ data }) => {
          this.zones = data.data;
        });
    },
    loadZone: async function (id: string): Promise<void> {
      const { $api } = useNuxtApp();

      $api()
        .compute.zones.get(id)
        .then(({ data }) => {
          this.zone = data;
        });
    },
    createZone: async function (data) {
      const { $api } = useNuxtApp();

      $api()
        .compute.zones.post(data)
        .then(({ data }) => {
          this.zone = data;
        });
    },
  },
});
