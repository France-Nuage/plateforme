import { ServiceMode } from "@/types";
import { useState } from "react";

export const useServiceMode = () => {
  const [mode, setMode] = useState<ServiceMode>(
    import.meta.env.VITE_ENVIRONMENT === "production"
      ? ServiceMode.Rpc
      : ServiceMode.Mock,
  );

  return { mode, setMode };
};
