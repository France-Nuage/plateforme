import { ComponentPropsWithoutRef, FunctionComponent } from "react";

export type ButtonProps = ComponentPropsWithoutRef<'button'>;

export const Button: FunctionComponent<ButtonProps> = ({ children, ...props }) => (
  <button {...props}>{children}</button>
)
