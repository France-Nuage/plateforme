/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type LinkedinSvgIconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function LinkedinSvgIcon(props: LinkedinSvgIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      fill={"none"}
      viewBox={"0 0 18 18"}
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
        fillRule={"evenodd"}
        clipRule={"evenodd"}
        d={
          "M2 0h14a2 2 0 012 2v14a2 2 0 01-2 2H2a2 2 0 01-2-2V2a2 2 0 012-2zm3 15a.5.5 0 00.5-.5v-7A.5.5 0 005 7H3.5a.5.5 0 00-.5.5v7a.5.5 0 00.5.5H5zm-.75-9a1.5 1.5 0 110-3 1.5 1.5 0 010 3zm10.25 9a.5.5 0 00.5-.5V9.9a3.1 3.1 0 00-2.72-3.14A3 3 0 009.5 8.1v-.6A.5.5 0 009 7H7.5a.5.5 0 00-.5.5v7a.5.5 0 00.5.5H9a.5.5 0 00.5-.5v-3.75a1.5 1.5 0 013 0v3.75a.5.5 0 00.5.5h1.5z"
        }
        fill={"currentColor"}
      ></path>
    </svg>
  );
}

export default LinkedinSvgIcon;
/* prettier-ignore-end */
