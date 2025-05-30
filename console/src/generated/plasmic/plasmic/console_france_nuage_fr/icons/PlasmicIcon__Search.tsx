/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { classNames } from '@plasmicapp/react-web';
import React from 'react';

export type SearchIconProps = React.ComponentProps<'svg'> & {
  title?: string;
};

export function SearchIcon(props: SearchIconProps) {
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
        'lucide lucide-search-icon lucide-search',
      )}
      viewBox={'0 0 24 24'}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path d={'M21 21l-4.34-4.34'}></path>

      <circle cx={'11'} cy={'11'} r={'8'}></circle>
    </svg>
  );
}

export default SearchIcon;
/* prettier-ignore-end */
