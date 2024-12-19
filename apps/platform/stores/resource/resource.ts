export const useResourceIAMStore = defineStore('resourceIAMStore', {
    state: () => ({
        members: [],
        policies: [],
    }),
    actions: {
        loadMembers: async function() {
            const { $api } = useNuxtApp()

            $api().iam.members.list().then(({ data }) => {
                this.members = data.bindings
            })
        },
        loadPolicies: async function() {
            const { $api } = useNuxtApp()

            $api().iam.roles.list().then(({ data }) => {
                this.members = data.bindings
            })
        }
    }
})