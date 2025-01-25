export const useServiceStore = defineStore("service", {
  state: () => ({
    services: [],
    service: null,
  }),
  actions: {
    loadServices: async (): Promise<void> => {
      const { $api } = useNuxtApp();

      $api()
        .services.list()
        .then(({ data }) => {
          this.services = data.data;
        });
    },
    loadService: async (id: string): Promise<void> => {
      const { $api } = useNuxtApp();

      $api()
        .services.get(id)
        .then(({ data }) => {
          this.service = data;
        });
    },
  },
});
