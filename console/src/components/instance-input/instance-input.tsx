import { FunctionComponent } from "react";
import { AppInput, AppRangeInput } from "@/components";
import { InstanceFormValue, MemoryBytes } from "@/types";

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
    <AppRangeInput
      label="CPU"
      max={16}
      milestones={[1, 4, 8, 12, 16]}
      min={1}
      name="instance-max-cpu-cores"
      onChange={(maxCpuCores) => onChange({ ...value, maxCpuCores })}
      value={value.maxCpuCores}
    />
    <AppRangeInput
      format={(value) => `${value / (2 ** 30)}GB`}
      label="Memory"
      max={MemoryBytes["32G"]}
      milestones={[MemoryBytes["1G"], MemoryBytes["4G"], MemoryBytes["8G"], MemoryBytes["16G"], MemoryBytes["32G"]]}
      min={MemoryBytes["1G"]}
      name="instance-max-memory-bytes"
      onChange={(maxMemoryBytes) => onChange({ ...value, maxMemoryBytes })}
      step={2 ** 30}
      value={value.maxMemoryBytes}
    />
  </div>
);
