/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
import { classNames } from '@plasmicapp/react-web';
import React from 'react';

export type GitLabLogoSvgIconProps = React.ComponentProps<'svg'> & {
  title?: string;
};

export function GitLabLogoSvgIcon(props: GitLabLogoSvgIconProps) {
  const { className, style, title, ...restProps } = props;
  return (
    <svg
      xmlns={'http://www.w3.org/2000/svg'}
      fill={'none'}
      viewBox={'0 0 40 40'}
      height={'1em'}
      className={classNames('plasmic-default__svg', className)}
      style={style}
      {...restProps}
    >
      {title && <title>{title}</title>}

      <path
        fill={'#E24329'}
        d={
          'm39.131 16.006-.054-.142L33.658 1.73a1.42 1.42 0 0 0-1.405-.894 1.415 1.415 0 0 0-1.299 1.048l-3.65 11.186h-14.8L8.84 1.884a1.4 1.4 0 0 0-.48-.73 1.445 1.445 0 0 0-1.668-.09 1.4 1.4 0 0 0-.556.673L.719 15.864l-.055.142a10.06 10.06 0 0 0 3.338 11.626l.017.012.05.038 8.252 6.178 4.082 3.089 2.483 1.877a1.67 1.67 0 0 0 2.024 0l2.483-1.877 4.082-3.09 8.3-6.207.022-.015a10.06 10.06 0 0 0 3.334-11.631'
        }
      ></path>

      <path
        fill={'#FC6D26'}
        d={
          'm39.131 16.006-.054-.142a18.3 18.3 0 0 0-7.284 3.275l-11.896 8.994 7.578 5.727 8.299-6.208.022-.015a10.06 10.06 0 0 0 3.335-11.631'
        }
      ></path>

      <path
        fill={'#FCA326'}
        d={
          'm12.32 33.86 4.083 3.089 2.483 1.877a1.67 1.67 0 0 0 2.024 0l2.483-1.877 4.082-3.09-7.579-5.726z'
        }
      ></path>

      <path
        fill={'#FC6D26'}
        d={
          'M8 19.14a18.3 18.3 0 0 0-7.28-3.276l-.056.142a10.06 10.06 0 0 0 3.338 11.626l.017.012.05.038 8.252 6.178 7.576-5.727z'
        }
      ></path>
    </svg>
  );
}

export default GitLabLogoSvgIcon;
/* prettier-ignore-end */
