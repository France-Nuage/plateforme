export default defineNuxtRouteMiddleware(async (to, from) => {
  // make authenticated state reactive
  const { authenticated } = storeToRefs(useAuthStore());
  // get token from cookies
  const token = useCookie("token");

  if (token.value) {
    // update the state to authenticated
    authenticated.value = true;
  }

  // if token exists and url is /login redirect to homepage
  if (token.value && to?.name === "auth-login") {
    return navigateTo("/");
  }

  // if token doesn't exist redirect to log in
  if (!token.value && to?.name !== "auth-login") {
    abortNavigation();
    return navigateTo("/auth/login");
  }

  // await authStore.loadAuthorizedRoutes()

  // try {
  //   const data = await authStore.navigationCheckingAuthorization(to.name)
  //   if ([403].includes(data.status)) {
  //     return navigateTo('/');
  //   }
  // } catch (e) {
  //   abortNavigation();
  //   return navigateTo('/');
  // }
});
