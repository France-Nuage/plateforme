import { User } from "@/types";
import { faker } from "@faker-js/faker/locale/nl_BE";

type RegistrationResponse = {
  id_token: string;
  session_state: null;
  access_token: string;
  token_type: 'Bearer';
  scope: 'email openid profile';
  profile: {
    aud: 'francenuage';
    exp: number;
    iat: number;
    iss: 'https://oidc';
  };
  expires_at: number;
};

export const createUser = (user: Partial<User>): Promise<RegistrationResponse> => fetch(`${process.env.OIDC_PROVIDER_URL}/api/users`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    email: user.email ?? faker.internet.email(),
    name: user.name ?? faker.person.fullName(),
    password: user.password ?? faker.internet.password(),
    username: user.username ?? faker.internet.username(),
  }),
}).then((response: Response) => response.json())
