import type { $Fetch, FetchOptions } from "ofetch";
import type { AuthenticationToken, User } from "@france-nuage/types";
export declare const securityRepository: (client: $Fetch) => {
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
  login: (
    body: Pick<User, "email" | "password">,
  ) => Promise<AuthenticationToken>;
  /**
   * Get the authenticated user info.
   */
  me: (options: FetchOptions<"json">) => Promise<User>;
  /**
   * Registers a new user.
   */
  register: (
    body: Pick<User, "email" | "firstname" | "lastname" | "password">,
  ) => Promise<{
    token: AuthenticationToken;
    user: User;
  }>;
};
//# sourceMappingURL=security.repository.d.ts.map
