/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type TwitterSvgIconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function TwitterSvgIcon(props: TwitterSvgIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      fill={"none"}
      viewBox={"0 0 20 17"}
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
          "M19.97 2.464a7.991 7.991 0 01-1.846 1.778v.55a12.19 12.19 0 01-3.621 8.69A12.178 12.178 0 015.773 17 12.217 12.217 0 01.33 15.74a.25.25 0 01-.15-.23v-.11a.26.26 0 01.26-.26 8.844 8.844 0 005.112-1.837 4.363 4.363 0 01-3.884-2.558.26.26 0 01.26-.36c.51.052 1.025.004 1.518-.14A4.355 4.355 0 010 6.44a.26.26 0 01.36-.26c.497.22 1.033.335 1.577.34A4.277 4.277 0 01.23 1.585a.54.54 0 01.91-.18A12.339 12.339 0 009.535 5.28a3.998 3.998 0 01-.12-1 4.352 4.352 0 017.499-2.937 8.674 8.674 0 002.387-.828.17.17 0 01.18 0 .17.17 0 010 .18 4.365 4.365 0 01-1.758 1.998 8.563 8.563 0 001.997-.48.16.16 0 01.18 0 .17.17 0 01.07.25z"
        }
        fill={"currentColor"}
      ></path>
    </svg>
  );
}

export default TwitterSvgIcon;
/* prettier-ignore-end */
