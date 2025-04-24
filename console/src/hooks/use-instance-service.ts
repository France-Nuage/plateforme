import { InstanceService } from "@/services/instance";
import { useTransport } from "./use-transport";
import { useEffect, useState } from "react";

export const useInstanceService = () => {
  const transport = useTransport();
  const [service, setService] = useState<InstanceService>();

  useEffect(() => {
    if (transport) {
      setService(new InstanceService(transport));
    }
  }, [transport]);

  return service;
};
