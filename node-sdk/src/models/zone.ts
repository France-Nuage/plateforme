/**
 * Represent a zone.
 */
export type Zone = {
  /**
   * The zone id.
   */
  id: string;

  /**
   * The zone name.
   */
  name: string;
};

export type ZoneFormValue = Pick<Zone, 'name'>;
