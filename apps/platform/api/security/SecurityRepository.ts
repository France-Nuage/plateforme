import { AxiosInstance } from 'axios';
import routes from './routes';
import type {ApiResponse} from "~/api/repositories/ApiResponse";

export interface AllowedLoginCredentials {
  username: string;
  password: string;
}

export interface AllowedRegisterCredentials {
  lastname: string;
  firstname: string;
  email: string;
  password: string;
  confirm_password: string;
}

export const SecurityRepository = function (client, config: Record<any, any>) {
  return {
    login: async (credentials: AllowedLoginCredentials) =>
      client(`/auth/token`, {
          // client_id: config.PASSPORT_CLIENT_ID,
          // client_secret: config.PASSPORT_CLIENT_SECRET,
          // grant_type: 'password',
          method: 'POST',
          body: { ...credentials },
        })
        .then((response) => {
          const token = useCookie('token'); // useCookie new hook in nuxt 3
          token.value = response.token?.token;
          return response;
        })
        .catch((e) => e.message)
    ,
    me: async () => {
        return client('/auth/me');
    },
    register: async (credentials: AllowedRegisterCredentials) =>
        client('/auth/register', {
            method: 'POST',
            body: {
                lastname: credentials.firstname,
                firstname: credentials.firstname,
                email: credentials.email,
                password: credentials.password,
            }
        })
    ,
    resetPasswordRequest: async (body) =>
        client(`/auth/reset-password-request`, {
            method: 'POST',
            body
        })
    ,
    resetPassword: async (body) => {
        return client(`/auth/reset-password`, {
            method: 'POST',
            body
        })
    }
  };
};

export default SecurityRepository;
