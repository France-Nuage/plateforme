import type { $Fetch } from "ofetch";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

/**
 * @deprecated
 */
interface PostOrganizationData {}

/**
 * @deprecated
 */
interface OrganizationResource {
  id: string;
  name: string;
  phone: string;
  fax: string;
  email: string;
  updated_at: string;
  created_at: string;
}

/**
 * @deprecated
 */
type PatchOrganizationData =
  | Partial<OrganizationResource>
  | { resultCode: string };

/**
 * @deprecated
 */
export const FolderRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<OrganizationResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/folders${apiCallParams}`, { method: "GET" });
    },
    get: async (
      folderId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<OrganizationResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/folders/${folderId}${apiCallParams}`);
    },
    post: async (
      body: PostOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/api/v1/folders`, { method: "POST", body });
    },
    patch: async (
      folderId: string,
      body: PatchOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/api/v1/folders/${folderId}`, { method: "PUT", body });
    },
  };
};
