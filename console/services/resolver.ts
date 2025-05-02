import { ServiceMode } from "@/types";
import { HypervisorService } from "./hypervisor.interface";
import { InstanceService } from "./instance.interface";
import { instanceMockService } from "./instance.mock";
import { hypervisorMockService } from "./hypervisor.mock";
import { hypervisorsRpcService } from "./hypervisor.rpc";
import { instanceRpcService } from "./instance.rpc";

type Services = {
  hypervisor: HypervisorService;
  instance: InstanceService;
};

export const services: Record<ServiceMode, Services> = {
  [ServiceMode.Mock]: {
    hypervisor: hypervisorMockService,
    instance: instanceMockService,
  },
  [ServiceMode.Rpc]: {
    hypervisor: hypervisorsRpcService,
    instance: instanceRpcService,
  },
};
