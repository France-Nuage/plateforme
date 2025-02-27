import type { $Fetch } from "nitropack";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

interface PostOrganizationData {}

interface OrganizationResource {
  id: string;
  name: string;
  phone: string;
  fax: string;
  email: string;
  updated_at: string;
  created_at: string;
}

type PatchOrganizationData =
  | Partial<OrganizationResource>
  | { resultCode: string };

export const InstanceRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<OrganizationResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/compute/instances${apiCallParams}`, { method: "GET" });
    },
    get: async (
      instanceId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<OrganizationResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/compute/instances/${instanceId}${apiCallParams}`);
    },
    post: async (
      body: PostOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/compute/instances`, { method: "POST", body });
    },
    patch: async (
      instanceId: string,
      body: PatchOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/compute/instances/${instanceId}`, {
        method: "PUT",
        body,
      });
    },
    getForecastPrice: async (
      body: PostOrganizationData,
    ): Promise<ApiResponse<OrganizationResource>> => {
      return client(`/compute/price`, { method: "POST", body });
    },
  };
};
