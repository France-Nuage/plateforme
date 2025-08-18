export const createUser = () => fetch(`${process.env.OIDC_PROVIDER_URL}/api/users`, {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    username: 'wile.coyote',
    email: 'wile.coyote@acme.org',
    name: 'Wile E. Coyote',
    password: 'killbipbip',
  }),
}).then((response) => response.json())
