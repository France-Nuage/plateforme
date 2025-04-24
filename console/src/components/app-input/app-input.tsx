import { ExclamationCircleIcon } from "@heroicons/react/16/solid";
import { clsx } from "clsx";
import { FunctionComponent } from "react";

export type AppInputProps = {
  error?: string;
  label: string;
  name: string;
  onChange: (value: string) => void;
  value: string;
};

export const AppInput: FunctionComponent<AppInputProps> = ({
  error,
  label,
  name,
  onChange,
  value,
  ...props
}) => (
  <div>
    <label
      htmlFor={name}
      className="block text-sm/6 font-medium text-gray-900"
    >{label}</label>
    <div className="mt-2 grid grid-cols-1">
      <input
        aria-invalid={!!error}
        aria-describedby={`${name}-error`}
        name={name}
        className={clsx(
          "col-start-1 row-start-1 block w-full rounded-md bg-white py-1.5 pl-3 pr-10 text-base outline-1 -outline-offset-1 focus:outline-2 focus:-outline-offset-2 sm:pr-9 sm:text-sm/6",
          error
            ? "text-red-900 outline-red-300 placeholder:text-red-300 focus:outline-red-600"
            : "text-gray-900 outline-gray-300 placeholder:text-gray-400 focus:outline-indigo-600",
        )}
        onChange={(event) => onChange(event.target.value)}
        value={value}
        {...props}
      />
      {!!error && (
        <ExclamationCircleIcon
          aria-hidden="true"
          className="pointer-events-none col-start-1 row-start-1 mr-3 size-5 self-center justify-self-end text-red-500 sm:size-4"
        />
      )}
    </div>
    {!!error && (
      <p id={`${name}-error`} className="mt-2 text-sm text-red-600">
        {error}
      </p>
    )}
  </div>
);
