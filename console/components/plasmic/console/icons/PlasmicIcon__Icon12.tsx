/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import React from "react";
import { classNames } from "@plasmicapp/react-web";

export type Icon12IconProps = React.ComponentProps<"svg"> & {
  title?: string;
};

export function Icon12Icon(props: Icon12IconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={"http://www.w3.org/2000/svg"}
      fill={"none"}
      stroke={"currentColor"}
      strokeWidth={"2"}
      strokeLinecap={"round"}
      strokeLinejoin={"round"}
      className={classNames(
        "plasmic-default__svg",
        className,
        "lucide lucide-image-icon lucide-image",
      )}
      viewBox={"0 0 24 24"}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <rect width={"18"} height={"18"} x={"3"} y={"3"} rx={"2"} ry={"2"}></rect>

      <circle cx={"9"} cy={"9"} r={"2"}></circle>

      <path d={"M21 15l-3.086-3.086a2 2 0 00-2.828 0L6 21"}></path>
    </svg>
  );
}

export default Icon12Icon;
/* prettier-ignore-end */
