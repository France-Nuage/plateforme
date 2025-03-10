import type { $Fetch } from "ofetch";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

/**
 * @deprecated
 */
interface PostProjectData {}

/**
 * @deprecated
 */
interface ProjectResource {
  id: string;
  name: string;
  updated_at: string;
  created_at: string;
}

/**
 * @deprecated
 */
type PatchProjectData = Partial<ProjectResource> | { resultCode: string };

/**
 * @deprecated
 */
export const ProjectRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<ProjectResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/projects${apiCallParams}`);
    },
    get: async (
      projectId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<ProjectResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/api/v1/projects/${projectId}${apiCallParams}`);
    },
    post: async (
      body: PostProjectData,
    ): Promise<ApiResponse<ProjectResource>> => {
      return client(`/api/v1/projects`, { method: "POST", body: body });
    },
    patch: async (
      projectId: string,
      body: PatchProjectData,
    ): Promise<ApiResponse<ProjectResource>> => {
      return client(`/api/v1/projects/${projectId}`, { method: "PUT", body });
    },
    delete: async (body: Array<string>): Promise<ApiResponse<any>> => {
      return client(`/api/v1/projects`, { method: "DELETE", body });
    },
  };
};
