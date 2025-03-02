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
export const PermissionRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<OrganizationResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/permissions${apiCallParams}`, { method: "GET" });
    },
    get: async (
      permissionId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<OrganizationResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/permissions/${permissionId}${apiCallParams}`);
    },
    post: async (
      body: PostOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/api/v1/permissions`, { method: "POST", body });
    },
    patch: async (
      permissionId: string,
      body: PatchOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/api/v1/permissions/${permissionId}`, {
        method: "PUT",
        body,
      });
    },
  };
};
