import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { useEffect, useState } from "react";

export const useTransport = () => {
  const [transport, setTransport] = useState<GrpcWebFetchTransport>();

  useEffect(() => {
    setTransport(
      new GrpcWebFetchTransport({
        baseUrl: import.meta.env.VITE_CONTROLPLANE_URL,
        format: "binary",
      }),
    );
  }, [setTransport]);

  return transport;
};
