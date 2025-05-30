/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type FacebookSvgIconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function FacebookSvgIcon(props: FacebookSvgIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      fill={"none"}
      viewBox={"0 0 10 18"}
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
          "M9.5 3h-3a1 1 0 00-1 1v3h4a.35.35 0 01.34.46l-.74 2.2a.5.5 0 01-.47.34H5.5v7.5a.5.5 0 01-.5.5H2.5a.5.5 0 01-.5-.5V10H.5a.5.5 0 01-.5-.5v-2A.5.5 0 01.5 7H2V4a4 4 0 014-4h3.5a.5.5 0 01.5.5v2a.5.5 0 01-.5.5z"
        }
        fill={"currentColor"}
      ></path>
    </svg>
  );
}

export default FacebookSvgIcon;
/* prettier-ignore-end */
