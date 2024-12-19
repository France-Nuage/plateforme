type Resource = { type: 'organization' | 'project' | 'folder', id?: string }
interface State {
    projects: any,
    project: any,
    organizations: any,
    organization: any,
    folders: any,
    folder: any,
    resource: Resource | null
}

export const useNavigationStore = defineStore('navigation', {
    state: (): State => ({
        projects: [],
        project: null,
        organizations: [],
        organization: null,
        folders: [],
        folder: null,
        resource: null
    }),
    actions: {
        loadProjects: async function () {
            const { $api } = useNuxtApp()

            return $api().projects.list().then((response) => {
                this.projects = response.data
                return response
            })
        },
        loadOrganizations: async function (queryParams: any = null) {
            const { $api } = useNuxtApp()

            return $api().organizations.list(queryParams).then((response) => {
                this.organizations = response.data
                // this.organization = response.data[0]

                // this.folders = this.organization.folders
                // this.folder = this.organization.folders[0]
                // this.projects = this.folder?.projects ? this.folder.projects[0] : []
                // this.project = this.projects.length > 0 ? this.projects[0] : null

                return response
            })
        },
        selectOrganization: async function (id: string) {
            const { $api } = useNuxtApp()

            return $api().organizations.get(id, { includes: 'folders.projects' }).then((response) => {
                this.folders = response.folders
                /**
                 * Default folder and project selected mode.
                 */
                // this.folder = response.folders[0]
                // this.projects = this.folder.projects ? this.folder.projects.map((item) => ({...item, folderId: this.folder.id})) : []
                // this.project = this.projects.length > 0 ? this.projects[0] : null

                /**
                 * No default folder and project selected mode.
                 */
                this.folder = null
                this.projects = []
                this.project = null
                this.organization = response

                this.resource = { type: 'organization', id: this.organization.id }
            })
        },
        selectFolder: async function (id: string) {
            const { $api } = useNuxtApp()

            return $api().folders.get(id, { includes: 'projects' }).then((response) => {
                this.folder = response
                this.projects = this.folder.projects ? this.folder.projects.map((item) => ({...item, folderId: this.folder.id})) : []

                /**
                 * Default project selected mode.
                 */
                // this.project = this.projects.length > 0 ? this.projects[0] : null

                /**
                 * No default project selected mode.
                 */
                this.project = null
                this.resource = { type: 'folder', id: this.folder.id }
            })
        },
        selectProject: async function (id: any) {
            this.project = this.projects.find((item: any) => item.id === id)

            this.resource = { type: 'project', id: this.project.id }
        },
        createResource: async function (body: any, resource: Resource) {
            const { $api } = useNuxtApp()
            const apis = {
                organization: $api().organizations,
                folder: $api().folders,
                project: $api().projects,
            }

            return apis[resource.type].post(body).then((response) => {
                return response
            })
        },
        updateResource: async function (id, body, resource: Resource) {
            const { $api } = useNuxtApp()
            const apis = {
                organization: $api().organizations,
                folder: $api().folders,
                project: $api().projects,
            }

            return apis[resource?.type].patch(id, body).then((response) => {
                return response
            })
        }
    }
})