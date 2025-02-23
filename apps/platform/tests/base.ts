import { test as base } from "@playwright/test";
import { LoginPage } from "./pages/login.page";
import { SignupPage } from "./pages/signup.page";
import { DashboardPage } from "./pages/dashboard.page";
import { ForgotPasswordPage } from "./pages/forgot-password.page";

// TODO: pull from global types
export type User = {
  email: string;
  password: string;
  lastname: string;
  firstname: string;
};

const user = {
  email: `${new Date().getTime()}-login.test@france-nuage.fr`,
  password: `${new Date().getTime()}-password`,
  firstname: "Coyote",
  lastname: "Acme",
};

type Fixtures = {
  pages: {
    dasboard: DashboardPage;
    forgotPassword: ForgotPasswordPage;
    login: LoginPage;
    signup: SignupPage;
  };
  user: User;
};

export const test = base.extend<Fixtures>({
  pages: async ({ page }, use) =>
    use({
      dasboard: new DashboardPage(page),
      forgotPassword: new ForgotPasswordPage(page, user),
      login: new LoginPage(page, user),
      signup: new SignupPage(page, user),
    }),
  user,
});

export { expect } from "@playwright/test";
