// eslint-disable-next-line @unicorn/filename-case
import { ModelQueryBuilderContract } from '@adonisjs/lucid/types/model'

class RequestQueryBuilder {
  private query: ModelQueryBuilderContract<any>
  private includes: string[] = []
  private includeOnly: string | null = null
  private filters: { [key: string]: any } = {}
  // private sorts: { [key: string]: any } = {}
  private pagination: { page: number | null; perPage: number | null } = {
    page: null,
    perPage: null,
  } // Valeurs par défaut

  constructor(query: ModelQueryBuilderContract<any>) {
    this.query = query
  }

  public withSorts() {
    return this
  }

  /**
   * Parse the `include` query parameter and set relationships to load
   * @param includeQueryParam
   */
  public withIncludes(includeQueryParam: string | Array<string> | undefined): this {
    if (includeQueryParam) {
      this.includes = Array.isArray(includeQueryParam) ? includeQueryParam : [includeQueryParam]
    }
    return this
  }

  /**
   * Parse the `include` query parameter and set relationships to load
   * @param includeQueryParam
   */
  public withIncludeOnly(includeQueryParam: string | undefined): this {
    if (includeQueryParam) {
      this.includeOnly = includeQueryParam
    }
    return this
  }

  /**
   * Set filters for the query
   * @param filters
   */
  public withFilters(filters: { [key: string]: any }): this {
    this.filters = filters
    return this
  }

  /**
   * Apply nested relationships (supports multiple levels)
   * @param relationPath
   * @param query
   */
  private preloadNestedRelations(
    relationPath: string,
    // query: ModelQueryBuilderContract<any>
  ): void {
    const relations = relationPath.split('.')

    const preloadRelation = (
      relation: string,
      nestedRelations: string[],
      builder: ModelQueryBuilderContract<any>
    ) => {
      if (nestedRelations.length > 0) {
        builder.preload(relation, (nestedQuery: ModelQueryBuilderContract<any>) => {
          preloadRelation(nestedRelations[0], nestedRelations.slice(1), nestedQuery)
        })
      } else {
        builder.preload(relation)
      }
    }

    preloadRelation(relations[0], relations.slice(1), this.query)
  }

  /**
   * Apply filters to the query
   */
  applyFilters() {
    const filters = this.filters
    if (filters) {
      Object.keys(filters).forEach((field) => {
        const filter = filters[field]
        Object.keys(filter).forEach((operator) => {
          const value = filter[operator]
          switch (operator) {
            case '_eq':
              this.query.orWhere(field, '=', value)
              break
            case '_ne':
              this.query.orWhere(field, '!=', value)
              break
            case '_lt':
              this.query.orWhere(field, '<', value)
              break
            case '_lte':
              this.query.orWhere(field, '<=', value)
              break
            case '_gt':
              this.query.orWhere(field, '>', value)
              break
            case '_gte':
              this.query.orWhere(field, '>=', value)
              break
            case '_like':
              this.query.orWhere(field, 'like', `%${value}%`)
              break
            case '_start_with':
              this.query.orWhere(field, 'like', `${value}%`)
              break
            case '_end_with':
              this.query.orWhere(field, 'like', `%${value}`)
              break
            case '_nnull':
              this.query.whereNotNull(field)
              break
            // Ajouter d'autres opérateurs si nécessaire
            default:
              break
          }
        })
      })
    }
    return this
  }

  /**
   * Set pagination for the query
   * @param page
   * @param perPage
   */
  public withPagination(page: number = 1, perPage: number = 10): this {
    this.pagination = { page, perPage }
    return this
  }

  /**
   * @param conditions
   */
  applyWhere(conditions: Array<any> = []) {
    // Vérifier si `conditions` est un tableau
    if (Array.isArray(conditions)) {
      conditions.forEach((condition) => {
        if (Array.isArray(condition) && condition.length === 3) {
          // Si la condition est un tableau [field, operator, value]
          this.query.where(condition[0], condition[1], condition[2])
        } else if (typeof condition === 'object' && condition.field && condition.value) {
          // Si la condition est un objet { field: 'name', value: 'John', operator: '=' }
          const operator = condition.operator || '='
          this.query.where(condition.field, operator, condition.value)
        }
      })
    }
    return this
  }

  async firstOrFail() {
    this.includes.forEach((relation) => {
      this.preloadNestedRelations(relation/*, this.query*/)
    })

    // Tenter de trouver le premier enregistrement ou échouer
    return await this.query.firstOrFail()
  }

  /**
   * Apply the necessary includes and filters to the query
   */
  public async apply(): Promise<ModelQueryBuilderContract<any>> {
    // if (!this.includeOnly) {
    // Apply includes
    this.includes.forEach((relation) => {
      this.preloadNestedRelations(relation/*, this.query*/)
    })
    // }

    // Apply filters
    this.applyFilters()

    if (this.includeOnly) {
      console.log('include only are not implemented')
    //   const result = await this.query.count('* as total')
    //   return { count: Number.parseInt(result[0]['$extras']['total']) }
    }

    if (this.pagination.page && this.pagination.perPage) {
      return await this.query.paginate(this.pagination.page, this.pagination.perPage)
    } else {
      return await this.query
    }
  }

  public async pojo() {
    return await this.query.pojo()
  }
}

export default RequestQueryBuilder
