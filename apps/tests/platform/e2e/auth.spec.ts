import { test } from "../base";

test.describe("Signup", () => {
  test("I can access the signup page from the login page", async ({
    pages,
  }) => {
    await pages.login.goto();
    await pages.login.signupLink.click();
    await pages.signup.assertLocation();
  });

  test("I can create a new account", async ({ pages }) => {
    await pages.signup.goto();
    await pages.signup.form.fill();
    await pages.signup.form.submit();
    await pages.dasboard.assertRedirectedTo();
  });
});

test.describe("Authentication", () => {
  test("I am redirected to the login page when not authenticated", async ({
    pages,
  }) => {
    await pages.dasboard.goto();
    await pages.login.assertRedirectedTo();
  });

  test("I can authenticate with valid credentials", async ({ pages }) => {
    await pages.login.goto();
    await pages.login.form.fill();
    await pages.login.form.submit();
    await pages.dasboard.assertRedirectedTo();
  });
});

test.describe("Password reinitialization", () => {
  test("I can access the password reinitialization page from the login page", async ({
    pages,
  }) => {
    await pages.login.goto();
    await pages.login.passwordForgottenLink.click();
    await pages.forgotPassword.assertRedirectedTo();
  });

  test("I can reinitialize my password without being authenticated", async ({
    pages,
  }) => {
    await pages.forgotPassword.goto();
    await pages.forgotPassword.form.fill();
    await pages.forgotPassword.form.submit();
    // TODO: this test is incomplete and does not test actual behavior
  });
});
