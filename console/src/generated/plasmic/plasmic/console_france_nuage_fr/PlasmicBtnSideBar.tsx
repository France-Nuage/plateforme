/* eslint-disable */
/* tslint:disable */
// @ts-nocheck
/* prettier-ignore-start */
/** @jsxRuntime classic */
/** @jsx createPlasmicElementProxy */
/** @jsxFrag React.Fragment */
// This class is auto-generated by Plasmic; please do not edit!
// Plasmic Project: aqt4bw2qhfo7d76ADZhQFo
// Component: A9xYAhwSD6i3
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
import sty from './PlasmicBtnSideBar.module.css';
import { useScreenVariants as useScreenVariantseEvMbXdv1ZEe } from './PlasmicGlobalVariant__Screen';
// plasmic-import: A9xYAhwSD6i3/css

import UserIcon from './icons/PlasmicIcon__User';
// plasmic-import: ohDidvG9XsCeFumugENU3J/projectcss
import projectcss from './plasmic.module.css';

// plasmic-import: HOWLf5KCV6xi/icon

createPlasmicElementProxy;

export type PlasmicBtnSideBar__VariantMembers = {
  isActive: 'isActive';
};
export type PlasmicBtnSideBar__VariantsArgs = {
  isActive?: SingleBooleanChoiceArg<'isActive'>;
};
type VariantPropType = keyof PlasmicBtnSideBar__VariantsArgs;
export const PlasmicBtnSideBar__VariantProps = new Array<VariantPropType>(
  'isActive',
);

export type PlasmicBtnSideBar__ArgsType = {
  nom?: string;
  children?: React.ReactNode;
  link?: string;
};
type ArgPropType = keyof PlasmicBtnSideBar__ArgsType;
export const PlasmicBtnSideBar__ArgProps = new Array<ArgPropType>(
  'nom',
  'children',
  'link',
);

export type PlasmicBtnSideBar__OverridesType = {
  root?: Flex__<'div'>;
  link?: Flex__<'a'>;
};

export interface DefaultBtnSideBarProps {
  nom?: string;
  children?: React.ReactNode;
  link?: string;
  isActive?: SingleBooleanChoiceArg<'isActive'>;
  className?: string;
}

const $$ = {};

function PlasmicBtnSideBar__RenderFunc(props: {
  variants: PlasmicBtnSideBar__VariantsArgs;
  args: PlasmicBtnSideBar__ArgsType;
  overrides: PlasmicBtnSideBar__OverridesType;
  forNode?: string;
}) {
  const { variants, overrides, forNode } = props;

  const args = React.useMemo(
    () =>
      Object.assign(
        {
          nom: 'txt',
          link: '#',
        },
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

  const stateSpecs: Parameters<typeof useDollarState>[0] = React.useMemo(
    () => [
      {
        path: 'isActive',
        type: 'private',
        variableType: 'variant',
        initFunc: ({ $props, $state, $queries, $ctx }) => $props.isActive,
      },
    ],
    [$props, $ctx, $refs],
  );
  const $state = useDollarState(stateSpecs, {
    $props,
    $ctx,
    $queries: {},
    $refs,
  });

  const globalVariants = ensureGlobalVariants({
    screen: useScreenVariantseEvMbXdv1ZEe(),
  });

  return (
    <Stack__
      as={'div'}
      data-plasmic-name={'root'}
      data-plasmic-override={overrides.root}
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
        sty.root,
        { [sty.rootisActive]: hasVariant($state, 'isActive', 'isActive') },
      )}
    >
      {renderPlasmicSlot({
        defaultContents: (
          <UserIcon
            className={classNames(projectcss.all, sty.svg__h3UDg)}
            role={'img'}
          />
        ),

        value: args.children,
      })}
      <PlasmicLink__
        data-plasmic-name={'link'}
        data-plasmic-override={overrides.link}
        className={classNames(
          projectcss.all,
          projectcss.a,
          projectcss.__wab_text,
          sty.link,
          { [sty.linkisActive]: hasVariant($state, 'isActive', 'isActive') },
        )}
        href={(() => {
          try {
            return $props.link;
          } catch (e) {
            if (
              e instanceof TypeError ||
              e?.plasmicType === 'PlasmicUndefinedDataError'
            ) {
              return 'https://www.plasmic.app/';
            }
            throw e;
          }
        })()}
        platform={'react'}
      >
        <React.Fragment>
          {(() => {
            try {
              return $props.nom;
            } catch (e) {
              if (
                e instanceof TypeError ||
                e?.plasmicType === 'PlasmicUndefinedDataError'
              ) {
                return '#';
              }
              throw e;
            }
          })()}
        </React.Fragment>
      </PlasmicLink__>
    </Stack__>
  ) as React.ReactElement | null;
}

const PlasmicDescendants = {
  root: ['root', 'link'],
  link: ['link'],
} as const;
type NodeNameType = keyof typeof PlasmicDescendants;
type DescendantsType<T extends NodeNameType> =
  (typeof PlasmicDescendants)[T][number];
type NodeDefaultElementType = {
  root: 'div';
  link: 'a';
};

type ReservedPropsType = 'variants' | 'args' | 'overrides';
type NodeOverridesType<T extends NodeNameType> = Pick<
  PlasmicBtnSideBar__OverridesType,
  DescendantsType<T>
>;
type NodeComponentProps<T extends NodeNameType> =
  // Explicitly specify variants, args, and overrides as objects
  {
    variants?: PlasmicBtnSideBar__VariantsArgs;
    args?: PlasmicBtnSideBar__ArgsType;
    overrides?: NodeOverridesType<T>;
  } & Omit<PlasmicBtnSideBar__VariantsArgs, ReservedPropsType> & // Specify variants directly as props
    // Specify args directly as props
    Omit<PlasmicBtnSideBar__ArgsType, ReservedPropsType> &
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
          internalArgPropNames: PlasmicBtnSideBar__ArgProps,
          internalVariantPropNames: PlasmicBtnSideBar__VariantProps,
        }),
      [props, nodeName],
    );
    return PlasmicBtnSideBar__RenderFunc({
      variants,
      args,
      overrides,
      forNode: nodeName,
    });
  };
  if (nodeName === 'root') {
    func.displayName = 'PlasmicBtnSideBar';
  } else {
    func.displayName = `PlasmicBtnSideBar.${nodeName}`;
  }
  return func;
}

export const PlasmicBtnSideBar = Object.assign(
  // Top-level PlasmicBtnSideBar renders the root element
  makeNodeComponent('root'),
  {
    // Helper components rendering sub-elements
    link: makeNodeComponent('link'),

    // Metadata about props expected for PlasmicBtnSideBar
    internalVariantProps: PlasmicBtnSideBar__VariantProps,
    internalArgProps: PlasmicBtnSideBar__ArgProps,
  },
);

export default PlasmicBtnSideBar;
/* prettier-ignore-end */
