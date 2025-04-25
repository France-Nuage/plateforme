import { FunctionComponent } from "react";
import { AppInput } from "@/components";
import { InstanceFormValue as InstanceFormValue } from "@/types";

export type InstanceInputProps = {
  onChange: (value: InstanceFormValue) => void;
  value: InstanceFormValue;
};

export const InstanceInput: FunctionComponent<InstanceInputProps> = ({
  onChange,
  value,
}) => (
  <div className="space-y-2">
    <AppInput
      label="Name"
      name="instance-name"
      onChange={(name) => onChange({ ...value, name })}
      value={value.name}
    />
    <AppInput
      label="Max CPU cores"
      name="instance-max-cpu-cores"
      onChange={(maxCpuCores) => onChange({ ...value, maxCpuCores })}
      value={value.maxCpuCores}
    />
  </div>
);
