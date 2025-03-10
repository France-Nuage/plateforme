export interface ApiIndexResponse<T> {
  meta: {
    total: number;
    perPage: number;
    currentPage: number;
    lastPage: number;
    firstPage: number;
    firstPageUrl: string;
    lastPageUrl: string;
    nextPageUrl?: number;
    previousPageUrl?: number;
  };
  data: T[];
}
