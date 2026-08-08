export default {
  /**
   * Base URL of the control plane (the confidential-client BFF).
   *
   * All authentication happens there: the browser is redirected to
   * `/auth/login` / `/auth/logout`, reads its identity from `/auth/me`, and
   * renews the session via `/auth/refresh`. The gRPC API is authenticated by
   * the httpOnly session cookie, never by a token held in JavaScript.
   */
  controlplane: import.meta.env.VITE_CONTROLPLANE_URL!,
};
