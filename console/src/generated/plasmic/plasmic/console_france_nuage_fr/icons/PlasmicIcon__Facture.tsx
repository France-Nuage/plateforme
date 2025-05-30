/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { classNames } from '@plasmicapp/react-web';
import React from 'react';

export type FactureIconProps = React.ComponentProps<'svg'> & {
  title?: string;
};

export function FactureIcon(props: FactureIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={'http://www.w3.org/2000/svg'}
      fill={'none'}
      stroke={'currentColor'}
      strokeWidth={'2'}
      strokeLinecap={'round'}
      strokeLinejoin={'round'}
      className={classNames(
        'plasmic-default__svg',
        className,
        'lucide lucide-receipt-text-icon lucide-receipt-text',
      )}
      viewBox={'0 0 24 24'}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path
        d={
          'M4 2v20l2-1 2 1 2-1 2 1 2-1 2 1 2-1 2 1V2l-2 1-2-1-2 1-2-1-2 1-2-1-2 1zm10 6H8m8 4H8m5 4H8'
        }
      ></path>
    </svg>
  );
}

export default FactureIcon;
/* prettier-ignore-end */
