import Organization from "~/middleware/organization";

interface State {
    organizations: Array<any>;
    organization: any
}

export const useOrganizationStore = defineStore('organization', {
    state: (): State => ({
        organizations: [],
        organization: null
    }),
    actions: {
        loadOrganizations: async function(queryParams: any = null) {
            const { $api } = useNuxtApp()
            return $api().organizations.list(queryParams).then(({ data, meta }) => {
                this.organizations = data
            })
        },
        loadOrganization: async function (id: string) {
            const { $api } = useNuxtApp()

            $api().organizations.get(id).then((response) => {
                this.organization = response
            })
        },
        createOrganization: function (formData) {
            const { $api } = useNuxtApp()

            return $api().organizations.post(formData).then((response) => {
                this.organization = response
                return response
            })
        }
    }
})