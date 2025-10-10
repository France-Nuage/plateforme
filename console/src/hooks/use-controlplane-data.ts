import { useEffect } from 'react';

import {
  fetchAllHypervisors,
  fetchAllInstances,
  fetchAllOrganizations,
  fetchAllProjects,
  fetchAllZeroTrustNetworkTypes,
  fetchAllZeroTrustNetworks,
  fetchAllZones,
} from '@/features';

import { useAppDispatch } from './use-app-dispatch';
import { useAppSelector } from './use-app-selector';

/**
 * Hook that loads controlplane data in a staged approach.
 *
 * Data loading happens in two stages:
 * 1. **Organizations First**: Loads organizations immediately when authenticated
 * 2. **Dependent Resources**: Loads all other resources once organizations are available
 *
 * This staged approach ensures proper data dependencies and prevents unnecessary
 * API calls when the user lacks organizational context.
 */
export function useControlplaneData() {
  const isAuthenticated = useAppSelector(
    (state) => !!state.authentication.token,
  );
  const organizations = useAppSelector(
    (state) => state.resources.organizations,
  );
  const dispatch = useAppDispatch();

  // Stage 1: Load organizations immediately when authenticated
  useEffect(() => {
    if (isAuthenticated) {
      dispatch(fetchAllOrganizations());
    }
  }, [dispatch, isAuthenticated]);

  // Stage 2: Load dependent resources once organizations are available
  useEffect(() => {
    if (isAuthenticated && organizations.length > 0) {
      dispatch(fetchAllHypervisors());
      dispatch(fetchAllInstances());
      dispatch(fetchAllProjects());
      dispatch(fetchAllZeroTrustNetworks());
      dispatch(fetchAllZeroTrustNetworkTypes());
      dispatch(fetchAllZones());
    }
  }, [dispatch, isAuthenticated, organizations]);
}
