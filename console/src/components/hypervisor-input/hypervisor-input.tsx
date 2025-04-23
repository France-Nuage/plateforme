import { AppInput } from "@/components";
import { Hypervisor } from "@/protocol";
import { FunctionComponent } from "react";

export type HypervisorInputProps = {
  onChange: (value: Hypervisor) => void;
  value: Hypervisor;
};

export const HypervisorInput: FunctionComponent<HypervisorInputProps> = ({
  onChange,
  value,
}) => (
  <div>
    <AppInput
      label="url"
      name="hypervisor-url"
      onChange={(url) => onChange({ ...value, url })}
      value={value.url}
    />
  </div>
);
