interface State {
    instances: Array<any>,
    instance: any,
}

export const useInstanceStore = defineStore('instance', {
    state: (): State => ({
        instances: [],
        instance: null,
    }),
    actions: {
        loadInstances: async function (queryParams?: any): Promise<void> {
            const { $api } = useNuxtApp()

            return $api().compute.instances.list(queryParams).then((response) => {
                this.instances = response.data
                return response
            })
        },
        loadInstance: async function (id: string, queryParams?: any): Promise<void> {
            const { $api } = useNuxtApp()

            return $api().compute.instances.get(id).then((response) => {
                this.instance = response

                return response
            })
        },
        createInstance: async function (data) {
            const { $api } = useNuxtApp()

            return $api().compute.instances.post(data).then(({ data }) => {
                this.instance = data
                return data
            })
        }
    }
})