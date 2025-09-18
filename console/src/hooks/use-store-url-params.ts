import { pickBy } from 'lodash';
import { useEffect } from 'react';
import { useSearchParams } from 'react-router';

import { setActiveState } from '@/features';

import { useAppDispatch } from './use-app-dispatch';
import { useAppSelector } from './use-app-selector';

/**
 * Hook that synchronizes application state with URL parameters.
 *
 * This hook replaces the previous useActiveParamsReconciliation hook with a
 * simplified approach. It performs two main functions:
 *
 * 1. Updates URL search parameters when active organization/project change
 * 2. Initializes active organization/project from available resources if none are set
 *
 * The hook ensures the URL reflects the current application state and provides
 * a fallback mechanism to set default active values when none exist.
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   useStoreUrlParams(); // Automatically syncs URL with app state
 *   return <div>...</div>;
 * }
 * ```
 */
export function useStoreUrlParams() {
  const activeOrganization = useAppSelector(
    (state) => state.application.activeOrganization,
  );
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const projects = useAppSelector((state) => state.resources.projects);
  const dispatch = useAppDispatch();
  const [searchParams, setSearchParams] = useSearchParams();

  useEffect(() => {
    const existing = Object.fromEntries([...searchParams.entries()]);
    setSearchParams(
      new URLSearchParams(
        pickBy(
          {
            ...existing,
            organization: activeOrganization?.id,
            project: activeProject?.id,
          },
          (value) => !!value,
        ) as Record<string, string>,
      ),
      { replace: true },
    );
  }, [activeOrganization, activeProject, searchParams, setSearchParams]);

  useEffect(() => {
    if (
      (!activeOrganization || !activeProject) &&
      organizations.length > 0 &&
      projects.length > 0
    ) {
      const activeOrganizationId =
        searchParams.get('organization') ?? organizations[0].id;
      const activeProjectId = searchParams.get('project') ?? projects[0].id;

      const activeOrganization = organizations.find(
        (organization) => organization.id === activeOrganizationId,
      );
      const activeProject = projects.find(
        (project) => project.id === activeProjectId,
      );

      if (!activeOrganization) {
        throw new Error(`no organization for id ${activeOrganizationId}`);
      }

      if (!activeProject) {
        throw new Error(
          `the organization ${activeOrganization.name} does not have any projects`,
        );
      }

      dispatch(setActiveState({ activeOrganization, activeProject }));
    }
  }, [activeOrganization, activeProject, dispatch, organizations, projects]);
}
