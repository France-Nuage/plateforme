import { useEffect } from 'react';
import { SetURLSearchParams, useSearchParams } from 'react-router';

import { setActiveOrganization, setActiveProject } from '@/features';
import { AppDispatch } from '@/store';
import { Organization, Project } from '@/types';

import { useAppDispatch, useAppSelector } from '.';

/**
 * Reconciliate active elements between URL and state.
 *
 * The application currently have 2 active parameters, scoped across the whole app:
 * - the active organization,
 * - the active project.
 *
 * This hook ensures those active elements are coherent and refer to existing data.
 */
export const useActiveParamsReconciliation = () => {
  // Select state portions
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization,
  );
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const projects = useAppSelector((state) => state.resources.projects);
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const defaultOrganization = useAppSelector(
    (state) => state.resources.organizations[0],
  );
  const defaultProject = useAppSelector((state) => state.resources.projects[0]);
  // Instantiate hooks
  const dispatch = useAppDispatch();
  const [searchParams, setSearchParams] = useSearchParams();
  const projectId = searchParams.get('project');
  const organizationId = searchParams.get('organization');

  useEffect(() => {
    reconciliateActiveProject(
      activeProject,
      defaultProject,
      projectId,
      projects,
      dispatch,
      setSearchParams,
    );
    reconciliateActiveOrganization(
      activeProject,
      activeOrganization,
      defaultOrganization,
      organizationId,
      organizations,
      dispatch,
      setSearchParams,
    );
  }, [
    activeOrganization,
    activeProject,
    defaultOrganization,
    defaultProject,
    dispatch,
    organizationId,
    projectId,
    organizations,
    projects,
    setSearchParams,
  ]);
};

function reconciliateActiveProject(
  activeProject: Project | undefined,
  defaultProject: Project,
  projectId: string | null,
  projects: Project[],
  dispatch: AppDispatch,
  setSearchParams: SetURLSearchParams,
) {
  // If neither the state, neither the url defines an active project, set the default one
  if (!projectId && !activeProject) {
    dispatch(setActiveProject(defaultProject));
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      project: defaultProject.id,
    }));
    return;
  }

  // Otherwise if there is an active project in the state but the url is missing it, add it to the url
  if (!projectId && activeProject) {
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      project: activeProject.id,
    }));
    return;
  }

  // Retrieve the project matching the url project id
  const project = projects.find((project) => project.id === projectId);

  // If the defined project does not map an existing project, revert to the default project
  if (!project) {
    dispatch(setActiveProject(defaultProject));
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      project: defaultProject.id,
    }));
    return;
  }

  // If the state active project does not match the url project, update the former
  if (!activeProject || activeProject.id !== projectId) {
    dispatch(setActiveProject(project));
    return;
  }
}

function reconciliateActiveOrganization(
  activeProject: Project | undefined,
  activeOrganization: Organization | undefined,
  defaultOrganization: Organization,
  organizationId: string | null,
  organizations: Organization[],
  dispatch: AppDispatch,
  setSearchParams: SetURLSearchParams,
) {
  // If neither the state, neither the url defines an active organization, set the default one
  if (!organizationId && !activeOrganization) {
    dispatch(setActiveOrganization(defaultOrganization));
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      organization: defaultOrganization.id,
    }));
    return;
  }

  // Otherwise if there is an active organization in the state but the url is missing it, add it to the url
  if (!organizationId && activeOrganization) {
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      organization: activeOrganization.id,
    }));
    return;
  }

  // Retrieve the organization matching the url project id
  const organization = organizations.find(
    (organization) => organization.id === organizationId,
  );

  // If the defined organization does not map an existing project, revert to the default organization
  if (!organization) {
    dispatch(setActiveOrganization(defaultOrganization));
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      organization: defaultOrganization.id,
    }));
    return;
  }

  // If the organization does not match the active project, reconciliate
  if (!!activeProject && activeProject.organizationId !== organization.id) {
    const organization = organizations.find(
      (organization) => organization.id === activeProject.organizationId,
    )!;
    dispatch(setActiveOrganization(organization));
    setSearchParams((previous) => ({
      ...Object.fromEntries(previous),
      organization: organization.id,
    }));
    return;
  }

  // If the state active organization does not match the url organization, update the former
  if (!activeOrganization || activeOrganization.id !== organizationId) {
    dispatch(setActiveOrganization(organization));
    return;
  }
}
