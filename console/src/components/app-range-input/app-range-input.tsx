import { FunctionComponent } from "react";

export type AppRangeInputProps = {
  format?: (value: number) => string;
  label: string;
  max?: number;
  milestones?: number[];
  min?: number;
  name: string;
  onChange: (value: number) => void;
  step?: number;
  value: number;
};

export const AppRangeInput: FunctionComponent<AppRangeInputProps> = ({
  format = (value) => String(value),
  label,
  max,
  milestones,
  min = 0,
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
    {milestones && max && (
      <div className="mx-2">
        <div className="w-full h-6 mt-1 relative">
          {milestones.map((milestone, i) => (
            <span
              key={i}
              className="absolute transform -translate-x-1/2 text-sm text-gray-500 dark:text-gray-400"
              style={{
                left: `${Math.max(0, Math.min(100, ((milestone - min) / (max - min)) * 100))}%`,
              }}
            >
              {format(milestone)}
            </span>
          ))}
        </div>
      </div>
    )}
  </div>
);
