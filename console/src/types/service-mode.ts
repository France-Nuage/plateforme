/**
 * The service mode variants.
 *
 * Represents the type of service to consume data from.
 */
export enum ServiceMode {
  /* Targets the gRPC services.
   *
   * Used in production for actual data, in system-tests for testing
   * interactions with the gRPC layer and in various environment to emulate 
   * production behavior.
   */
  Rpc = "rpc",

  /**
   * Targets the mock services.
   *
   * Mocks are used in the plasmic studio for building the interface without
   * depending on external services, as well as tests that don't require
   * cross-service interactions.
   */
  Mock = "mock",
}
