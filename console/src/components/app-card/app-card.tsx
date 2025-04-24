import { clsx } from "clsx";
import { ComponentPropsWithoutRef, FunctionComponent, ReactNode } from "react";

export type AppCardProps = {
  children: ReactNode;
  className?: string;
  header?: ReactNode;
  padded?: boolean;
} & ComponentPropsWithoutRef<"div">;

export const AppCard: FunctionComponent<AppCardProps> = ({
  children,
  className,
  header,
  padded = true,
}) => (
  <div
    className={clsx(
      className,
      "divide-y divide-gray-200 overflow-hidden rounded-lg bg-white shadow",
    )}
  >
    {header && <div className="bg-gray-50 px-4 py-5 sm:px-6">{header}</div>}
    <div className={clsx({ "px-4 py-5 sm:p-6": padded })}>{children}</div>
  </div>
);
