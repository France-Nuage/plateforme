export const useRoleStore = defineStore("role", {
  state: () => ({
    roles: [],
    role: null,
    currentRole: null,
  }),
  actions: {
    loadRoles: async function (): Promise<void> {
      const { $api } = useNuxtApp();

      $api()
        .roles.list()
        .then((data) => {
          console.log(data);
          this.roles = data;
        });
    },
    loadRole: async function (id: string): Promise<void> {
      const { $api } = useNuxtApp();

      $api()
        .roles.get(id)
        .then(({ data }) => {
          this.role = data;
        });
    },
    createRole: async function (data) {
      const { $api } = useNuxtApp();

      $api()
        .roles.post(data)
        .then(({ data }) => {
          this.role = data;
        });
    },
  },
});
