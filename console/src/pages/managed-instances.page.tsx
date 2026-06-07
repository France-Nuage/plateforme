import { Button, EmptyState, HStack, Heading } from '@chakra-ui/react';
import { FunctionComponent, useEffect, useMemo } from 'react';
import { HiSquare3Stack3D } from 'react-icons/hi2';
import { Link } from 'react-router';

import { ManagedInstanceTable } from '@/components';
import { fetchManagedInstances } from '@/features';
import { useAppDispatch, useAppSelector, usePolling } from '@/hooks';
import { MANAGED_INSTANCE_POLL_INTERVAL_MS } from '@/services';
import { Routes } from '@/types';

/**
 * Managed instances page.
 *
 * Lists every managed service instance belonging to the active project,
 * polling the list while the user stays on the page so that provisioning
 * transitions are visible without a manual refresh.
 */
export const ManagedInstancesPage: FunctionComponent = () => {
  const dispatch = useAppDispatch();
  const activeProject = useAppSelector(
    (state) => state.application.activeProject,
  );
  const instances = useAppSelector((state) => state.managedServices.instances);
  const services = useAppSelector((state) => state.managedServices.services);

  const scopedInstances = useMemo(
    () =>
      instances.filter((instance) => instance.projectId === activeProject?.id),
    [instances, activeProject],
  );

  useEffect(() => {
    if (activeProject) {
      dispatch(fetchManagedInstances(activeProject.id));
    }
  }, [dispatch, activeProject]);

  usePolling(
    () => {
      if (activeProject) {
        dispatch(fetchManagedInstances(activeProject.id));
      }
    },
    MANAGED_INSTANCE_POLL_INTERVAL_MS,
    !!activeProject,
  );

  return (
    <>
      <HStack>
        <Heading flexGrow={1} whiteSpace="nowrap">
          Instances de services managés
        </Heading>
        <Button asChild>
          <Link to={Routes.ManagedServices}>Déployer un service</Link>
        </Button>
      </HStack>

      {scopedInstances.length === 0 ? (
        <EmptyState.Root>
          <EmptyState.Content>
            <EmptyState.Indicator>
              <HiSquare3Stack3D />
            </EmptyState.Indicator>
            <EmptyState.Title>Aucune instance déployée</EmptyState.Title>
            <EmptyState.Description>
              Déployez une application prête à l'emploi depuis le catalogue de
              services managés. Chaque instance est isolée et entièrement gérée
              pour vous.
            </EmptyState.Description>
            <Button asChild colorPalette="blue">
              <Link to={Routes.ManagedServices}>Déployer un service</Link>
            </Button>
          </EmptyState.Content>
        </EmptyState.Root>
      ) : (
        <ManagedInstanceTable instances={scopedInstances} services={services} />
      )}
    </>
  );
};
