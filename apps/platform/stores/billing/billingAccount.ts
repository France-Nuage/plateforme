export const useAccountStore = defineStore('billingAccount', {
    state: () => ({
        billingAccounts: [],
        billingAccount: null
    }),
    actions: {
        loadBillingAccounts: async (): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().billingAccounts.list().then(({ data }) => {
                this.billingAccounts = data.data
            })
        },
        loadBillingAccount: async (id: string): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().billingAccounts.get(id).then(({ data }) => {
                this.billingAccount = data
            })
        },
        createBillingAccount: async function (data) {
            const { $api } = useNuxtApp()

            return $api().billingAccounts.post(data).then(({ data }) => {
                this.billingAccount = data
            })
        }
    }
})