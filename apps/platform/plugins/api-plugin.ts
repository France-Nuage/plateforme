import repositories from '../api';
import { nanoid } from 'nanoid';
import { useAlerts } from "~/stores/alert";

export default defineNuxtPlugin((app) => {
  const config = useRuntimeConfig();
  const alerts = useAlerts();
  const route = app._route
  const router = useRouter()
  const client = $fetch.create({
    baseURL: 'http://localhost:3333/api/v1',
    timeout: 5000,
    retry: 0,
    retryDelay: 5000,
    // withCredentials: true,
    headers: {
      'Content-Type': 'application/json',
      Accept: 'application/json',
      'x-request-id': nanoid(),
    },
    onRequest({ request, options, error }) {
      const newConfig = { ...config };
      const token = useCookie('token');
      const keySet = token.value;
      const organizationId = useCookie('organization')

      if (token.value === undefined) return newConfig;

      if (keySet) {
        options.headers.set('Authorization', `Bearer ${keySet}`);
      }

      options.headers.set('x-organization-id', organizationId.value);
      if ('project-id' in route.query) {
        options.headers.set('x-project-id', route.query['project-id']);
      }
    },
    onResponseError(error) {
      const { request, response, options } = error
      // Log error
      console.log(
          "[fetch response error]",
          request,
          response.status,
          response.body
      );

      if (response.status === 401) {
        router.push('/auth/login')
      }

      if (response._data.errors) {
        response._data.errors.forEach((item) => {
          alerts.add({
            title: response.statusText,
            description: item.message,
          })
        })
      } else {
        alerts.add({
          title: response._data.name,
          description: response._data.message,
        })
      }

      return error
    },
  })

  return {
    provide: {
      api: () => {
        return {
          ...repositories(client, config),
        };
      },
    },
  };
});
