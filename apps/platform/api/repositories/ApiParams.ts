interface Filter<F> {
  fieldName: F;
  value: string | number;
}

export type Order = "+" | "-";

interface Sort<S> {
  fieldName: S;
  order?: Order;
}

export interface AllowedParams<F = null, S = null, I = null> {
  sort?: Array<Sort<S>>;
  filter?: Array<Filter<F>>;
  include?: Array<I>;
}
