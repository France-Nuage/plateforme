export const organizationsRepository = (client) => ({
  /**
   * @inheritdoc
   */
  list: (options) => client(`/api/v1/organizations`, options),
  /**
   * @inheritdoc
   */
  read: (id, options) => client(`/api/v1/organizations/${id}`, options),
  /**
   * @inheritdoc
   */
  create: (body, options) =>
    client(`/api/v1/organizations`, {
      method: "POST",
      body: body,
      ...options,
    }),
  /**
   * @inheritdoc
   */
  update: (id, body, options) =>
    client(`/api/v1/organizations/${id}`, {
      method: "PATCH",
      body: body,
      ...options,
    }),
  /**
   * @inheritdoc
   */
  delete: (id, options) => client(`/api/v1/organizations/${id}`, options),
});
//# sourceMappingURL=organizations.repository.js.map
