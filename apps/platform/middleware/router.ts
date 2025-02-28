function addQueryParams(to, params) {
  return {
    ...to,
    query: {
      ...to.query,
      ...params,
    },
  };
}

export default defineNuxtRouteMiddleware(async (to, from) => {
  const navigationStore = useNavigationStore();
  const { organization, folder, project } = storeToRefs(navigationStore);
  const { selectOrganization, selectFolder, selectProject } = navigationStore;

  const storeOrganizationId = organization.value?.id || null;
  const storeFolderId = folder.value?.id || null;
  const storeProjectId = project.value?.id || null;

  const toQuery = to.query || {};
  const fromQuery = from.query || {};

  const nextOrganizationId = toQuery.organization || storeOrganizationId;
  const nextFolderId = toQuery.folder || storeFolderId;
  const nextProjectId = toQuery.project || storeProjectId;

  const queryParams = {};

  if (nextOrganizationId && nextOrganizationId !== toQuery.organization) {
    queryParams.organization = nextOrganizationId;
  }

  if (
    nextFolderId &&
    nextFolderId !== toQuery.folder &&
    !(toQuery.organization !== fromQuery.organization)
  ) {
    queryParams.folder = nextFolderId;
  }

  if (
    nextProjectId &&
    nextProjectId !== toQuery.project &&
    !(toQuery.folder !== fromQuery.folder) &&
    !(toQuery.organization !== fromQuery.organization)
  ) {
    queryParams.project = nextProjectId;
  }

  const shouldRedirect = Object.keys(queryParams).length > 0;

  if (shouldRedirect) {
    return navigateTo(addQueryParams(to, queryParams));
  }

  // Charger les données dans le store après s'être assuré que l'URL est correcte
  if (nextOrganizationId) {
    await selectOrganization(nextOrganizationId);
  }

  if (nextFolderId && !(toQuery.organization !== fromQuery.organization)) {
    await selectFolder(nextFolderId);
  }

  if (
    nextProjectId &&
    !(toQuery.folder !== fromQuery.folder) &&
    !(toQuery.organization !== fromQuery.organization)
  ) {
    await selectProject(nextProjectId);
  }
});
