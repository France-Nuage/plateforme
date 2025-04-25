import { FunctionComponent } from "react";

export type AppRangeInputProps = {
  label: string;
  max?: number;
  milestones?: number;
  min?: number;
  name: string;
  onChange: (value: number) => void;
  step?: number;
  value: number;
};

export const AppRangeInput: FunctionComponent<AppRangeInputProps> = ({
  label,
  max,
  milestones,
  min,
  name,
  onChange,
  step,
  value,
}) => (
  <div>
    <label
      htmlFor={name}
      className="block mb-2 text-sm font-medium text-gray-900 dark:text-white"
    >
      {label}
    </label>
    <input
      className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer dark:bg-gray-700"
      max={max}
      min={min}

      name={name}
      onChange={(event) => onChange(Number(event.target.value))}
      step={step}
      type="range"
      value={value}
    />
    {milestones && (<div className="w-full flex justify-between px-1">
      {Array.from({ length: milestones }, (_, i) => (
        <span key={i} className="text-sm text-gray-500 dark:text-gray-400 w-0">
          {Number((min! + i * ((max! - min!) / (milestones - 1))).toFixed(2))}
        </span>
      ))}

    </div>)}
  </div>
);
