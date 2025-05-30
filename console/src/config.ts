export default {
  controlplane: import.meta.env.VITE_CONTROLPLANE_URL!,
  oidc: {
    clientId: import.meta.env.VITE_OIDC_CLIENT_ID!,
    name: import.meta.env.VITE_OIDC_PROVIDER_NAME!,
    url: import.meta.env.VITE_OIDC_PROVIDER_URL!,
  },
};
