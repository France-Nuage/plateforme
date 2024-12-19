export const useFolderStore = defineStore('folder', {
    state: () => ({
        folders: [],
        folder: null,
        currentFolder: null,
    }),
    actions: {
        loadFolders: async (): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().folders.list().then(({ data }) => {
                this.folders = data.data
            })
        },
        loadFolder: async (id: string): Promise<void> => {
            const { $api } = useNuxtApp()

            $api().folders.get(id).then(({ data }) => {
                this.folder = data
            })
        },
        createFolder: async function (data) {
            const { $api } = useNuxtApp()

            $api().folders.post(data).then(({ data }) => {
                this.folder = data
            })
        }
    }
})