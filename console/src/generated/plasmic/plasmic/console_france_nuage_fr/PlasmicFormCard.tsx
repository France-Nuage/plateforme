/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
/** @jsxRuntime classic */
/** @jsx createPlasmicElementProxy */
/** @jsxFrag React.Fragment */
// This class is auto-generated by Plasmic; please do not edit!
// Plasmic Project: aqt4bw2qhfo7d76ADZhQFo
// Component: MRElL_DPUhFE
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

import plasmic_antd_5_hostless_css from '../antd_5_hostless/plasmic.module.css';
// plasmic-import: aqt4bw2qhfo7d76ADZhQFo/projectcss
import sty from './PlasmicFormCard.module.css';
import { useScreenVariants as useScreenVariantseEvMbXdv1ZEe } from './PlasmicGlobalVariant__Screen';
// plasmic-import: ohDidvG9XsCeFumugENU3J/projectcss
import projectcss from './plasmic.module.css';

// plasmic-import: MRElL_DPUhFE/css

createPlasmicElementProxy;

export type PlasmicFormCard__VariantMembers = {};
export type PlasmicFormCard__VariantsArgs = {};
type VariantPropType = keyof PlasmicFormCard__VariantsArgs;
export const PlasmicFormCard__VariantProps = new Array<VariantPropType>();

export type PlasmicFormCard__ArgsType = { content?: React.ReactNode };
type ArgPropType = keyof PlasmicFormCard__ArgsType;
export const PlasmicFormCard__ArgProps = new Array<ArgPropType>('content');

export type PlasmicFormCard__OverridesType = {
  form2?: Flex__<'div'>;
};

export interface DefaultFormCardProps {
  content?: React.ReactNode;
  className?: string;
}

const $$ = {};

function PlasmicFormCard__RenderFunc(props: {
  variants: PlasmicFormCard__VariantsArgs;
  args: PlasmicFormCard__ArgsType;
  overrides: PlasmicFormCard__OverridesType;
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
    <Stack__
      as={'div'}
      data-plasmic-name={'form2'}
      data-plasmic-override={overrides.form2}
      data-plasmic-root={true}
      data-plasmic-for-node={forNode}
      hasGap={true}
      className={classNames(
        projectcss.all,
        projectcss.root_reset,
        projectcss.plasmic_default_styles,
        projectcss.plasmic_mixins,
        projectcss.plasmic_tokens,
        plasmic_antd_5_hostless_css.plasmic_tokens,
        sty.form2,
      )}
    >
      {renderPlasmicSlot({
        defaultContents: null,
        value: args.content,
      })}
    </Stack__>
  ) as React.ReactElement | null;
}

const PlasmicDescendants = {
  form2: ['form2'],
} as const;
type NodeNameType = keyof typeof PlasmicDescendants;
type DescendantsType<T extends NodeNameType> =
  (typeof PlasmicDescendants)[T][number];
type NodeDefaultElementType = {
  form2: 'div';
};

type ReservedPropsType = 'variants' | 'args' | 'overrides';
type NodeOverridesType<T extends NodeNameType> = Pick<
  PlasmicFormCard__OverridesType,
  DescendantsType<T>
>;
type NodeComponentProps<T extends NodeNameType> =
  // Explicitly specify variants, args, and overrides as objects
  {
    variants?: PlasmicFormCard__VariantsArgs;
    args?: PlasmicFormCard__ArgsType;
    overrides?: NodeOverridesType<T>;
  } & Omit<PlasmicFormCard__VariantsArgs, ReservedPropsType> & // Specify variants directly as props
    // Specify args directly as props
    Omit<PlasmicFormCard__ArgsType, ReservedPropsType> &
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
          internalArgPropNames: PlasmicFormCard__ArgProps,
          internalVariantPropNames: PlasmicFormCard__VariantProps,
        }),
      [props, nodeName],
    );
    return PlasmicFormCard__RenderFunc({
      variants,
      args,
      overrides,
      forNode: nodeName,
    });
  };
  if (nodeName === 'form2') {
    func.displayName = 'PlasmicFormCard';
  } else {
    func.displayName = `PlasmicFormCard.${nodeName}`;
  }
  return func;
}

export const PlasmicFormCard = Object.assign(
  // Top-level PlasmicFormCard renders the root element
  makeNodeComponent('form2'),
  {
    // Helper components rendering sub-elements

    // Metadata about props expected for PlasmicFormCard
    internalVariantProps: PlasmicFormCard__VariantProps,
    internalArgProps: PlasmicFormCard__ArgProps,
  },
);

export default PlasmicFormCard;
/* prettier-ignore-end */
