/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { classNames } from '@plasmicapp/react-web';
import React from 'react';

export type DevSquareIconProps = React.ComponentProps<'svg'> & {
  title?: string;
};

export function DevSquareIcon(props: DevSquareIconProps) {
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
        'lucide lucide-square-terminal-icon lucide-square-terminal',
      )}
      viewBox={'0 0 24 24'}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path d={'M7 11l2-2-2-2m4 6h4'}></path>

      <rect width={'18'} height={'18'} x={'3'} y={'3'} rx={'2'} ry={'2'}></rect>
    </svg>
  );
}

export default DevSquareIcon;
/* prettier-ignore-end */
