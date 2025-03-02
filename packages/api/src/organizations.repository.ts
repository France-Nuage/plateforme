import { Organization } from "@france-nuage/types";
import type { Repository } from "./types";

export const organizationsRepository: Repository<Organization> = (client) => ({
  /**
   * @inheritdoc
   */
  list: (options) => client(`/api/v1/organizations`, options),

  /**
   * @inheritdoc
   */
  read: (id, options) => client(`/api/v1/organizations/${id}`, options),

  /**
   * @inheritdoc
   */
  create: (body, options) =>
    client(`/api/v1/organizations`, {
      method: "POST",
      body: body as Record<string, any>,
      ...options,
    }),

  /**
   * @inheritdoc
   */
  update: (id, body, options) =>
    client(`/api/v1/organizations/${id}`, {
      method: "PATCH",
      body: body as Record<string, any>,
      ...options,
    }),

  /**
   * @inheritdoc
   */
  delete: (id, options) => client(`/api/v1/organizations/${id}`, options),
});
