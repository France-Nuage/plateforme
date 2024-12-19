interface UserPayloadInterface {
  id?: string;
  lastname?: string;
  firstname?: string;
  email: string;
  password: string;
}

interface UserInterface {
  id: string;
  lastname: string;
  firstname: string;
  email: string;
  object: string;
  updatedAt: string;
  createdAt: string;
}


export const useAuthStore = defineStore('auth', {
  state: (): { authenticated: boolean, me: UserInterface | null, routes: any } => ({
    authenticated: false,
    me: null,
    routes: []
  }),
  actions: {
    loadMe: async function () {
      const { $api } = useNuxtApp()
      return $api().security.me().then(response => {
        this.me = response
      })
    },
    subscribe: async function (payload: UserPayloadInterface): Promise<void> {
      const { $api } = useNuxtApp()
      return $api().security.register(payload).then((data) => {
        if (data.token) {
          const token = useCookie('token');
          token.value = data?.token?.token;
          this.authenticated = true;
        }
      })
    },
    authenticate: async function ({ email, password }: UserPayloadInterface) {
      const { $api } = useNuxtApp()
      return $api().security.login({ email, password }).then((response) => {
        if (response.data.token) {
          const token = useCookie('token');

          token.value = response.data?.token?.token;
          this.authenticated = true;
        }

        return response;
      })
    },
    resetPasswordRequest: async function ({ email }: { email: string }): Promise<void> {
      const { $api } = useNuxtApp()
      return $api().security.resetPasswordRequest({ email: email })
    },
    resetPassword: async function (payload: { password: string, token: string }): Promise<void> {
      const { $api } = useNuxtApp()
      return $api().security.resetPassword(payload)
    },
    update: async function (payload: UserPayloadInterface): Promise<void> {
      const { $api } = useNuxtApp()
      return $api().members.patch(this.me.id, payload).then((response) => {
        this.me = response
      })
    },
    logUserOut: function () {
      const token = useCookie('token');
      this.authenticated = false;
      token.value = null;
    },
  },
});