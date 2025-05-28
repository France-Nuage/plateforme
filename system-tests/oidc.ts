export const createUser = () => fetch('https://oidc:4000/api/users', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({}),
}).then((response) => response.json())
