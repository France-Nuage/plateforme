export const securityRepository = (client) => ({
  /**
   * Logs the user in.
   *
   * This function creates a new authentication token. The caller is then
   * responsible for applying the token as an authentication header to
   * performing authenticated actions.
   *
   * **Usage**
   *
   * ```js
   * const user = { email: 'wile.coyote@france-nuage.fr', password: 'password' }
   * const { token } = await securityRepository.login(user)
   * const info = await securityRepository.me({
   *  headers: { 'Authorization': `Bearer ${ token }` }
   * })
   * ```
   */
  login: (body) => client(`/api/v1/login`, { method: "POST", body }),
  /**
   * Get the authenticated user info.
   */
  me: (options) => client(`/api/v1/me`, options),
  /**
   * Registers a new user.
   */
  register: (body) => client(`/api/v1/auth/register`, { method: "POST", body }),
});
//# sourceMappingURL=security.repository.js.map
