import { HypervisorService } from "@/services/hypervisor";
import { useTransport } from "./use-transport";
import { useEffect, useState } from "react";

export const useHypervisorService = () => {
  const transport = useTransport();
  const [service, setService] = useState<HypervisorService>();
  useEffect(() => {
    if (transport) {
      setService(new HypervisorService(transport));
    }
  }, [transport]);
  return service;
};
