import { ComponentPropsWithoutRef, FunctionComponent } from "react";
import { Size } from "../../types";
import clsx from "clsx";

export type AppButtonProps = { size?: Size } & ComponentPropsWithoutRef<"button">;

export const AppButton: FunctionComponent<AppButtonProps> = ({
  children,
  size = "md",
  ...props
}) => {
  return (
    <button
      className={clsx(
        "bg-indigo-600 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600",
        sizeClassName[size],
      )}
      {...props}
    >
      {children}
    </button>
  );
};

const sizeClassName: Record<Size, string> = {
  sm: "rounded px-2 py-1",
  md: "rounded-md px-2.5 py-1.5",
  lg: "rounded-md px-3.5 py-2.5",
};
