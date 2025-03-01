import { Organization } from "@france-nuage/types";
import type { Repository } from "./types";

export const organizationsRepository: Repository<Organization, "list"> = (
  client,
  config,
) => ({
  /**
   * @inheritdoc
   */
  list: (options) => client(`/organizations`, options),
});
