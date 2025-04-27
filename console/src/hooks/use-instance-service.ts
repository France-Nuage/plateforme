import { InstanceRpcService } from "@/services";
import { useTransport } from "./use-transport";
import { useEffect, useState } from "react";

export const useInstanceService = () => {
  const transport = useTransport();
  const [service, setService] = useState<InstanceRpcService>();

  useEffect(() => {
    if (transport) {
      setService(new InstanceRpcService(transport));
    }
  }, [transport]);

  return service;
};
