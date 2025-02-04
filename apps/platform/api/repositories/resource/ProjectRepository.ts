import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";

interface PostProjectData {}

interface ProjectResource {
  id: string;
  name: string;
  updated_at: string;
  created_at: string;
}

type PatchProjectData = Partial<ProjectResource> | { resultCode: string };

export const ProjectRepository = function (
  client: any,
  config: Record<any, any>,
) {
  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<ProjectResource[]>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/projects${apiCallParams}`);
    },
    get: async (
      projectId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<ProjectResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(`/projects/${projectId}${apiCallParams}`);
    },
    post: async (
      body: PostProjectData,
    ): Promise<ApiResponse<ProjectResource>> => {
      return client(`/projects`, { method: "POST", body: body });
    },
    patch: async (
      projectId: string,
      body: PatchProjectData,
    ): Promise<ApiResponse<ProjectResource>> => {
      return client(`/projects/${projectId}`, { method: "PUT", body });
    },
    delete: async (body: Array<string>): Promise<ApiResponse<any>> => {
      return client(`/projects`, { method: "DELETE", body });
    },
  };
};
