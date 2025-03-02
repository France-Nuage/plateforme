import { $Fetch, FetchOptions } from "ofetch";
import { ApiIndexResponse } from "./api-index-response";

/**
 * Represents an API resource repository implementation.
 *
 * A *repository* provides CRUD operations for a given resource `U`. It accepts
 * two generics:
 *  - `Resource`, representing the underlying resource exposed through the
 *    API endpoint.
 *  - `Operations`, representing the operations provided by the endpoint.
 *
 * The `Operations` generic notably allows to define a *partial repository for
 * resources that should not have a given operation, e.g. a `delete` on a
 * `User`.
 *
 * **Usage**
 *
 * ```js
 * const folderRepository: Repository<Folder, 'list'> = (client, config) => ({
 *   list: (options) => client(`/organizations`, options)
 * });
 * ```
 */
export type Repository<
  Resource,
  Operations extends "list" | "read" | "create" | "update" | "delete" =
    | "list"
    | "read"
    | "create"
    | "update"
    | "delete",
> = (
  client: $Fetch,
  config: Record<any, any>,
) => Pick<
  {
    /**
     * Gets a listing of the resource.
     */
    list: (
      options?: FetchOptions<"json", Resource>,
    ) => Promise<ApiIndexResponse<Resource>>;

    /**
     * Gets the resource matching the given resource.
     */
    read: (
      id: string,
      options?: FetchOptions<"json", Resource>,
    ) => Promise<Resource>;

    /**
     * Creates a new resource.
     */
    create: <U = Partial<Resource>>(
      body: U,
      options?: FetchOptions<"json", Resource>,
    ) => Promise<Resource>;

    /**
     * Updates the given resource.
     */
    update: <U = Partial<Resource>>(
      id: string,
      body: U,
      options?: FetchOptions<"json", Resource>,
    ) => Promise<Resource>;

    /**
     * Deletes the given resource.
     */
    delete: (
      id: string,
      options?: FetchOptions<"json", Resource>,
    ) => Promise<void>;
  },
  Operations
>;
