/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
/** @jsxRuntime classic */
/** @jsx createPlasmicElementProxy */
/** @jsxFrag React.Fragment */
// This class is auto-generated by Plasmic; please do not edit!
// Plasmic Project: aqt4bw2qhfo7d76ADZhQFo
// Component: p-wD1wOjqe01
import {
  get as $stateGet,
  set as $stateSet,
  Flex as Flex__,
  MultiChoiceArg,
  PlasmicDataSourceContextProvider as PlasmicDataSourceContextProvider__,
  PlasmicIcon as PlasmicIcon__,
  PlasmicImg as PlasmicImg__,
  PlasmicLink as PlasmicLink__,
  PlasmicPageGuard as PlasmicPageGuard__,
  SingleBooleanChoiceArg,
  SingleChoiceArg,
  Stack as Stack__,
  StrictProps,
  Trans as Trans__,
  classNames,
  createPlasmicElementProxy,
  deriveRenderOpts,
  ensureGlobalVariants,
  generateOnMutateForSpec,
  generateStateOnChangeProp,
  generateStateOnChangePropForCodeComponents,
  generateStateValueProp,
  hasVariant,
  initializeCodeComponentStates,
  initializePlasmicStates,
  makeFragment,
  omit,
  pick,
  renderPlasmicSlot,
  useCurrentUser,
  useDollarState,
  usePlasmicTranslator,
  useTrigger,
  wrapWithClassName,
} from '@plasmicapp/react-web';
import {
  DataCtxReader as DataCtxReader__,
  useDataEnv,
  useGlobalActions,
} from '@plasmicapp/react-web/lib/host';
// plasmic-import: eEvMBXdv1ZEe/globalVariant

import '@plasmicapp/react-web/lib/plasmic.css';
import * as React from 'react';

import ConsoleBreadcrumb from '../../ConsoleBreadcrumb';
// plasmic-import: PEgOxZRQTA56/component
import NavigationDrawer from '../../NavigationDrawer';
import plasmic_antd_5_hostless_css from '../antd_5_hostless/plasmic.module.css';
// plasmic-import: aqt4bw2qhfo7d76ADZhQFo/projectcss
import sty from './PlasmicConsoleLayout.module.css';
// plasmic-import: WgBKdGHdVFBy/component

import { useScreenVariants as useScreenVariantseEvMbXdv1ZEe } from './PlasmicGlobalVariant__Screen';
// plasmic-import: ohDidvG9XsCeFumugENU3J/projectcss
import projectcss from './plasmic.module.css';

// plasmic-import: p-wD1wOjqe01/css

createPlasmicElementProxy;

export type PlasmicConsoleLayout__VariantMembers = {};
export type PlasmicConsoleLayout__VariantsArgs = {};
type VariantPropType = keyof PlasmicConsoleLayout__VariantsArgs;
export const PlasmicConsoleLayout__VariantProps = new Array<VariantPropType>();

export type PlasmicConsoleLayout__ArgsType = { children?: React.ReactNode };
type ArgPropType = keyof PlasmicConsoleLayout__ArgsType;
export const PlasmicConsoleLayout__ArgProps = new Array<ArgPropType>(
  'children',
);

export type PlasmicConsoleLayout__OverridesType = {
  root?: Flex__<'div'>;
  consoleBreadcrumb?: Flex__<typeof ConsoleBreadcrumb>;
  navigationDrawer?: Flex__<typeof NavigationDrawer>;
};

export interface DefaultConsoleLayoutProps {
  children?: React.ReactNode;
  className?: string;
}

const $$ = {};

function PlasmicConsoleLayout__RenderFunc(props: {
  variants: PlasmicConsoleLayout__VariantsArgs;
  args: PlasmicConsoleLayout__ArgsType;
  overrides: PlasmicConsoleLayout__OverridesType;
  forNode?: string;
}) {
  const { variants, overrides, forNode } = props;

  const args = React.useMemo(
    () =>
      Object.assign(
        {},
        Object.fromEntries(
          Object.entries(props.args).filter(([_, v]) => v !== undefined),
        ),
      ),
    [props.args],
  );

  const $props = {
    ...args,
    ...variants,
  };

  const $ctx = useDataEnv?.() || {};
  const refsRef = React.useRef({});
  const $refs = refsRef.current;

  const globalVariants = ensureGlobalVariants({
    screen: useScreenVariantseEvMbXdv1ZEe(),
  });

  return (
    <div
      data-plasmic-name={'root'}
      data-plasmic-override={overrides.root}
      data-plasmic-root={true}
      data-plasmic-for-node={forNode}
      className={classNames(
        projectcss.all,
        projectcss.root_reset,
        projectcss.plasmic_default_styles,
        projectcss.plasmic_mixins,
        projectcss.plasmic_tokens,
        plasmic_antd_5_hostless_css.plasmic_tokens,
        sty.root,
      )}
    >
      <ConsoleBreadcrumb
        data-plasmic-name={'consoleBreadcrumb'}
        data-plasmic-override={overrides.consoleBreadcrumb}
        className={classNames('__wab_instance', sty.consoleBreadcrumb)}
      />

      <div className={classNames(projectcss.all, sty.freeBox__vp0Ai)}>
        <NavigationDrawer
          data-plasmic-name={'navigationDrawer'}
          data-plasmic-override={overrides.navigationDrawer}
          className={classNames('__wab_instance', sty.navigationDrawer)}
        />

        <Stack__
          as={'div'}
          hasGap={true}
          className={classNames(projectcss.all, sty.freeBox__h0SBk)}
        >
          {renderPlasmicSlot({
            defaultContents: null,
            value: args.children,
          })}
        </Stack__>
      </div>
    </div>
  ) as React.ReactElement | null;
}

const PlasmicDescendants = {
  root: ['root', 'consoleBreadcrumb', 'navigationDrawer'],
  consoleBreadcrumb: ['consoleBreadcrumb'],
  navigationDrawer: ['navigationDrawer'],
} as const;
type NodeNameType = keyof typeof PlasmicDescendants;
type DescendantsType<T extends NodeNameType> =
  (typeof PlasmicDescendants)[T][number];
type NodeDefaultElementType = {
  root: 'div';
  consoleBreadcrumb: typeof ConsoleBreadcrumb;
  navigationDrawer: typeof NavigationDrawer;
};

type ReservedPropsType = 'variants' | 'args' | 'overrides';
type NodeOverridesType<T extends NodeNameType> = Pick<
  PlasmicConsoleLayout__OverridesType,
  DescendantsType<T>
>;
type NodeComponentProps<T extends NodeNameType> =
  // Explicitly specify variants, args, and overrides as objects
  {
    variants?: PlasmicConsoleLayout__VariantsArgs;
    args?: PlasmicConsoleLayout__ArgsType;
    overrides?: NodeOverridesType<T>;
  } & Omit<PlasmicConsoleLayout__VariantsArgs, ReservedPropsType> & // Specify variants directly as props
    // Specify args directly as props
    Omit<PlasmicConsoleLayout__ArgsType, ReservedPropsType> &
    // Specify overrides for each element directly as props
    Omit<
      NodeOverridesType<T>,
      ReservedPropsType | VariantPropType | ArgPropType
    > &
    // Specify props for the root element
    Omit<
      Partial<React.ComponentProps<NodeDefaultElementType[T]>>,
      ReservedPropsType | VariantPropType | ArgPropType | DescendantsType<T>
    >;

function makeNodeComponent<NodeName extends NodeNameType>(nodeName: NodeName) {
  type PropsType = NodeComponentProps<NodeName> & { key?: React.Key };
  const func = function <T extends PropsType>(
    props: T & StrictProps<T, PropsType>,
  ) {
    const { variants, args, overrides } = React.useMemo(
      () =>
        deriveRenderOpts(props, {
          name: nodeName,
          descendantNames: PlasmicDescendants[nodeName],
          internalArgPropNames: PlasmicConsoleLayout__ArgProps,
          internalVariantPropNames: PlasmicConsoleLayout__VariantProps,
        }),
      [props, nodeName],
    );
    return PlasmicConsoleLayout__RenderFunc({
      variants,
      args,
      overrides,
      forNode: nodeName,
    });
  };
  if (nodeName === 'root') {
    func.displayName = 'PlasmicConsoleLayout';
  } else {
    func.displayName = `PlasmicConsoleLayout.${nodeName}`;
  }
  return func;
}

export const PlasmicConsoleLayout = Object.assign(
  // Top-level PlasmicConsoleLayout renders the root element
  makeNodeComponent('root'),
  {
    // Helper components rendering sub-elements
    consoleBreadcrumb: makeNodeComponent('consoleBreadcrumb'),
    navigationDrawer: makeNodeComponent('navigationDrawer'),

    // Metadata about props expected for PlasmicConsoleLayout
    internalVariantProps: PlasmicConsoleLayout__VariantProps,
    internalArgProps: PlasmicConsoleLayout__ArgProps,
  },
);

export default PlasmicConsoleLayout;
/* prettier-ignore-end */
