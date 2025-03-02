import { Organization } from "@france-nuage/types";
import type { Repository } from "./types";

export const organizationsRepository: Repository<
  Organization,
  "list" | "read"
> = (client) => ({
  /**
   * @inheritdoc
   */
  list: (options) => client(`/organizations`, options),

  /**
   * @inheritdoc
   */
  read: (id, options) => client(`/organizations/${id}`, options),
});
