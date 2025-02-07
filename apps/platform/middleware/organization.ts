import { useNavigationStore } from "~/stores/navigation";

export default defineNuxtRouteMiddleware(async (to, from) => {
  const { loadOrganizations } = useNavigationStore();
  const { organizations } = storeToRefs(useNavigationStore());
  await loadOrganizations({ includes: "folders.projects" });

  if (!organizations.value === undefined || organizations.value.length === 0) {
    abortNavigation();
    return navigateTo("/onboarding/new");
  }
});
