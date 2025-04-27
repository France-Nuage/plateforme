import { useEffect, useState } from "react";
import { useServiceMode } from "./use-service-mode";
import {
  hypervisorMockService,
  HypervisorService,
  hypervisorsRpcService,
  instanceMockService,
  instanceRpcService,
  InstanceService,
} from "@/services";
import { ServiceMode } from "@/types";

type Services = {
  hypervisor: HypervisorService;
  instance: InstanceService;
};

const services: Record<ServiceMode, Services> = {
  [ServiceMode.Mock]: {
    hypervisor: hypervisorMockService,
    instance: instanceMockService,
  },
  [ServiceMode.Rpc]: {
    hypervisor: hypervisorsRpcService,
    instance: instanceRpcService,
  },
};

export const useServices = () => {
  const { mode } = useServiceMode();

  const [state, setState] = useState<Services>(services[mode]);

  useEffect(() => {
    setState(services[mode]);
  }, [mode]);

  return state;
};
