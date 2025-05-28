/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type IconLeft3IconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function IconLeft3Icon(props: IconLeft3IconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      xmlnsXlink={"http://www.w3.org/1999/xlink"}
      fill={"none"}
      viewBox={"0 0 20 20"}
      height={"1em"}
      className={classNames("plasmic-default__svg", className)}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <g clipPath={"url(#a)"}>
        <path
          stroke={"currentColor"}
          strokeLinecap={"round"}
          strokeLinejoin={"round"}
          strokeOpacity={".6"}
          strokeWidth={"1.667"}
          d={
            "m18.148 8.981-7.13-7.13a1.44 1.44 0 0 0-2.037 0l-7.13 7.13a1.44 1.44 0 0 0 0 2.037l7.13 7.13a1.44 1.44 0 0 0 2.037 0l7.13-7.13a1.44 1.44 0 0 0 0-2.037"
          }
        ></path>
      </g>

      <defs>
        <clipPath id={"a"}>
          <path fill={"currentColor"} d={"M0 0h20v20H0z"}></path>
        </clipPath>
      </defs>
    </svg>
  );
}

export default IconLeft3Icon;
/* prettier-ignore-end */
