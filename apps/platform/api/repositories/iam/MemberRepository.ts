import type { $Fetch } from "nitropack";
import { parseUri } from "../../parsers/url";
import type { AllowedParams } from "./../ApiParams";
import type { ApiResponse } from "./../ApiResponse";
import { useNavigationStore } from "~/stores/navigation";

interface PostUserData {}

interface UserResource {
  id: string;
  name: string;
  establishmentIdentifier: string;
  updatedAt: string;
  createdAt: string;
}

type PatchUserData = Partial<UserResource> | { resultCode: string };

export const IAMMemberRepository = function (
  client: $Fetch,
  config: Record<any, any>,
) {
  const { resource } = storeToRefs(useNavigationStore());

  if (!resource) {
    return;
  }

  const resourceValue: { type: string; id: string } = resource.value;

  return {
    list: async (
      params?: AllowedParams<any, null, null>,
    ): Promise<ApiResponse<any>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(
        `/${resourceValue.type}/${resourceValue.id}/iam/members${apiCallParams}`,
      );
    },
    get: async (
      userId: string,
      params?: AllowedParams<null, null, null>,
    ): Promise<ApiResponse<UserResource>> => {
      const apiCallParams = params ? parseUri(params) : "";
      return client(
        `/${resourceValue.type}/${resourceValue.id}/iam/members/${userId}${apiCallParams}`,
      );
    },
    post: async (body: PostUserData): Promise<ApiResponse<UserResource>> => {
      return client(`/${resourceValue.type}/${resourceValue.id}/iam/members`, {
        method: "POST",
        body,
      });
    },
    patch: async (
      userId: string,
      body: PatchUserData,
    ): Promise<ApiResponse<UserResource>> => {
      return client(
        `/${resourceValue.type}/${resourceValue.id}/iam/members/${userId}`,
        { method: "PUT", body },
      );
    },
    delete: async (ids: Array<string>): Promise<Awaited<any>[]> => {
      return Promise.all(
        ids.map(
          async (id: string) =>
            await client(
              `/${resourceValue.type}/${resourceValue.id}/iam/members/${id}`,
              { method: "DELETE" },
            ),
        ),
      );
    },
  };
};
