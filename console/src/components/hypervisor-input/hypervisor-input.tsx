import { AppInput } from "@/components";
import { Hypervisor } from "@/types";
import { FunctionComponent } from "react";

export type HypervisorInputProps = {
  onChange: (value: Omit<Hypervisor, "id">) => void;
  value: Omit<Hypervisor, "id">;
};

export const HypervisorInput: FunctionComponent<HypervisorInputProps> = ({
  onChange,
  value,
}) => (
  <div className="space-y-2">
    <AppInput
      label="Url"
      name="hypervisor-url"
      onChange={(url) => onChange({ ...value, url })}
      value={value.url}
    />
    <AppInput
      label="Authorization Token"
      name="hypervisor-authorization-token"
      onChange={(authorizationToken) =>
        onChange({ ...value, authorizationToken })
      }
      value={value.authorizationToken!}
    />
    <AppInput
      label="Storage Name"
      name="hypervisor-storage-name"
      onChange={(storageName) => onChange({ ...value, storageName })}
      value={value.storageName}
    />
  </div>
);
