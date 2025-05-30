/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { classNames } from '@plasmicapp/react-web';
import React from 'react';

export type HomeIconProps = React.ComponentProps<'svg'> & {
  title?: string;
};

export function HomeIcon(props: HomeIconProps) {
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
        'lucide lucide-house-icon lucide-house',
      )}
      viewBox={'0 0 24 24'}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path d={'M15 21v-8a1 1 0 00-1-1h-4a1 1 0 00-1 1v8'}></path>

      <path
        d={
          'M3 10a2 2 0 01.709-1.528l7-5.999a2 2 0 012.582 0l7 5.999A2 2 0 0121 10v9a2 2 0 01-2 2H5a2 2 0 01-2-2z'
        }
      ></path>
    </svg>
  );
}

export default HomeIcon;
/* prettier-ignore-end */
