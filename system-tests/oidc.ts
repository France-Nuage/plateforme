export const createUser = () => fetch(`${process.env.OIDC_PROVIDER_URL}/api/users`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({}),
}).then((response) => response.json())
