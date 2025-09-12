/**
 * The number of milliseconds to delay in the error debounce.
 */
export const ERROR_DEBOUNCE_WAIT = 500;

/**
 * Session storage key for persisting OIDC access tokens.
 *
 * This key is used to store and retrieve JWT access tokens in the browser's
 * session storage, enabling authentication persistence across browser tabs
 * while ensuring tokens are cleared when the browser session ends.
 *
 * The key follows the `frn.oidc.*` naming convention for France Nuage OIDC
 * related storage keys.
 */
export const ACCESS_TOKEN_SESSION_STORAGE_KEY = 'frn.oidc.token';
