export const useProjectStore = defineStore('project', {
    state: () => ({
        projects: [],
        project: null,
        currentProject: null,
    }),
    actions: {
        loadProjects: async (): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().projects.list().then(({ data }) => {
                this.projects = data.data
            })
        },
        loadProject: async (id: string): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().projects.get(id).then(({ data }) => {
                this.project = data
            })
        },
        createProject: async function (data) {
            const { $api } = useNuxtApp()

            $api().projects.post(data).then(({ data }) => {
                this.project = data
            })
        }
    }
})