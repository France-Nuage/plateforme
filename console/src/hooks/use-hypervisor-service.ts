import { HypervisorRpcService } from "@/services";
import { useTransport } from "./use-transport";
import { useEffect, useState } from "react";

export const useHypervisorService = () => {
  const transport = useTransport();
  const [service, setService] = useState<HypervisorRpcService>();
  useEffect(() => {
    if (transport) {
      setService(new HypervisorRpcService(transport));
    }
  }, [transport]);
  return service;
};
