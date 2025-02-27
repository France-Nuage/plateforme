exports[`auth_login > login user 1`] = `{
  "email": "jhon.doe@france-nuage.com",
  "firstname": "Jhon",
  "lastname": "Doe",
  "object": "user",
}`

exports[`auth_login > login user fail 422 error 1`] = `{
  "errors": [
    {
      "field": "email",
      "message": "The email field must be defined",
      "rule": "required",
    },
    {
      "field": "password",
      "message": "The password field must be defined",
      "rule": "required",
    },
  ],
}`

