exports[`auth_register > register user 1`] = `{
  "email": undefined,
  "firstname": undefined,
  "lastname": undefined,
  "object": undefined,
}`

exports[`auth_register > register user failed 422 error with short password 1`] = `{
  "errors": [
    {
      "field": "password",
      "message": "The password field must have at least 8characters",
      "meta": {
        "min": 8,
      },
      "rule": "minLength",
    },
  ],
}`

exports[`auth_register > register user failed 422 error with missed information 1`] = `{
  "errors": [
    {
      "field": "lastname",
      "message": "The lastname field must be defined",
      "rule": "required",
    },
    {
      "field": "firstname",
      "message": "The firstname field must be defined",
      "rule": "required",
    },
  ],
}`

exports[`auth_register > register user failed 422 error with bad format user 1`] = `{
  "errors": [
    {
      "field": "lastname",
      "message": "The lastname field must be defined",
      "rule": "required",
    },
    {
      "field": "firstname",
      "message": "The firstname field must be defined",
      "rule": "required",
    },
  ],
}`

exports[`auth_register > register user fail 422 error with email already exist 1`] = `{
  "errors": [
    {
      "field": "email",
      "message": "The email has already been taken",
      "rule": "database.unique",
    },
  ],
}`

