export const useBillingAccountStore = defineStore("billingAccount", {
  state: () => ({
    billingAccounts: [],
    billingAccount: null,
  }),
  actions: {
    loadBillingAccounts: async (): Promise<void> => {
      const { $api } = useNuxtApp();

      $api()
        .billing.accounts.list()
        .then(({ data }) => {
          this.billingAccounts = data.data;
        });
    },
    loadBillingAccount: async (id: string): Promise<void> => {
      const { $api } = useNuxtApp();

      $api()
        .billing.accounts.get(id)
        .then(({ data }) => {
          this.billingAccount = data;
        });
    },
    createBillingAccount: async function (data) {
      const { $api } = useNuxtApp();

      return $api()
        .billing.accounts.post(data)
        .then(({ data }) => {
          this.billingAccount = data;
        });
    },
  },
});
