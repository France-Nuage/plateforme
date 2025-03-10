import { expect, test } from "../../base.js";

test.describe("GET /api/v1/auth/me", () => {
  test("I can retrieve my info as a user", async ({ actingWith }) => {
    const { request, user } = await actingWith();
    const response = await request.get("/api/v1/auth/me");
    const result = await response.json();

    expect(response.status()).toBe(200);
    expect(Object.keys(result).sort()).toEqual([
      "createdAt",
      "email",
      "firstname",
      "id",
      "lastname",
      "object",
      "updatedAt",
    ]);
    expect(result).toMatchObject({
      email: user.email,
      firstname: user.firstname,
      lastname: user.lastname,
      object: "user",
    });
  });

  test("I cannot retrieve my info without being authenticated", async ({
    request,
  }) => {
    const response = await request.get("/api/v1/auth/me");
    const result = await response.json();

    expect(response.status()).toBe(401);
    expect(result).toEqual({ errors: [{ message: "Unauthorized access" }] });
  });
});
