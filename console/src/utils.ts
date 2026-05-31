/**
 * Extracts a human-readable message from an unknown thrown value.
 *
 * RTK Query thunks reject with a `SerializedError` (a plain object carrying a
 * `message` field), not a native `Error`, so `instanceof Error` alone is not
 * enough. Falls back to `fallback` when no usable message is present.
 */
export function getErrorMessage(
  error: unknown,
  fallback = 'Une erreur est survenue.',
): string {
  if (error instanceof Error) {
    return error.message || fallback;
  }
  if (typeof error === 'object' && error !== null && 'message' in error) {
    const { message } = error as { message: unknown };
    if (typeof message === 'string' && message.length > 0) {
      return message;
    }
  }
  return fallback;
}
