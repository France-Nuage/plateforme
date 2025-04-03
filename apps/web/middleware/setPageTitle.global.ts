export default defineNuxtRouteMiddleware((to) => {
  if (to.name) {
    const pageName = to.name.toString();
    const formattedTitle = pageName
      .replace(/-/g, " ") // Remplace les tirets par des espaces
      .replace(/(^\w|\s\w)/g, (m) => m.toUpperCase()); // Majuscule au début de chaque mot

    useHead({
      title: `${formattedTitle} - france-nuage`, // Ajoute le titre formaté
    });
  }
});
