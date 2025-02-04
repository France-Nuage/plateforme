interface State {
  instances: Array<any>;
  instance: any;
  price: null | { total: number; ram?: number; cpu?: number; disk?: number };
}

export const useInstanceStore = defineStore("instance", {
  state: (): State => ({
    instances: [],
    instance: null,
    price: null,
  }),
  actions: {
    loadInstances: async function (queryParams?: any): Promise<void> {
      const { $api } = useNuxtApp();

      return $api()
        .compute.instances.list(queryParams)
        .then((response) => {
          this.instances = response.data;
          return response;
        });
    },
    loadInstance: async function (
      id: string,
      queryParams?: any,
    ): Promise<void> {
      const { $api } = useNuxtApp();

      return $api()
        .compute.instances.get(id)
        .then((response) => {
          this.instance = response;

          return response;
        });
    },
    createInstance: async function (data) {
      const { $api } = useNuxtApp();

      return $api()
        .compute.instances.post(data)
        .then(({ data }) => {
          this.instance = data;
          return data;
        });
    },
    getForecastPrice: async function (data) {
      const { $api } = useNuxtApp();

      return $api()
        .compute.instances.getForecastPrice(data)
        .then((data) => {
          console.log(this.price);
          this.price = data;
          return data;
        });
    },
  },
});
