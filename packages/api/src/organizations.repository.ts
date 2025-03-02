import { Organization } from "@france-nuage/types";
import type { Repository } from "./types";

export const organizationsRepository: Repository<
  Organization,
  "list" | "read"
> = (client) => ({
  /**
   * @inheritdoc
   */
  list: (options) => client(`/api/v1/organizations`, options),

  /**
   * @inheritdoc
   */
  read: (id, options) => client(`/api/v1/organizations/${id}`, options),
});
