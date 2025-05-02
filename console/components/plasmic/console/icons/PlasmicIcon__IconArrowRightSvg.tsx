/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type IconArrowRightSvgIconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function IconArrowRightSvgIcon(props: IconArrowRightSvgIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      fill={"none"}
      viewBox={"0 0 24 24"}
      height={"1em"}
      width={"1em"}
      style={{
        fill: "currentcolor",

        ...(style || {}),
      }}
      className={classNames("plasmic-default__svg", className)}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path
        d={
          "M11.06 16.14a.5.5 0 01-.71 0l-.2-.2a.49.49 0 01-.15-.36V8.42a.49.49 0 01.15-.36l.2-.2a.5.5 0 01.71 0l3.79 3.79a.48.48 0 010 .7l-3.79 3.79z"
        }
        fill={"currentColor"}
      ></path>
    </svg>
  );
}

export default IconArrowRightSvgIcon;
/* prettier-ignore-end */
