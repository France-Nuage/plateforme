<template>
  <nuxt-layout>
    <div class="relative isolate px-6 pt-14 lg:px-8 bg-gray-50">
      <div class="mx-auto max-w-4xl pt-20 pb-16 sm:pt-28 lg:pt-32">
        <!-- Titre Principal -->
        <h1 class="text-3xl font-bold text-gray-900 mb-8">
          Documentation France-Nuage
        </h1>

        <!-- Introduction -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Introduction
          </h2>
          <p class="text-gray-700 mb-4">
            Bienvenue sur la documentation de France-Nuage. Ici, vous trouverez
            toutes les informations nécessaires pour configurer, déployer et
            gérer vos ressources avec France-Nuage. Que vous soyez un
            utilisateur débutant ou un administrateur expérimenté, cette
            documentation est conçue pour vous guider à travers chaque étape du
            processus.
          </p>
          <p class="text-gray-700">
            Utilisez le menu ci-dessous pour naviguer vers les différentes
            sections de la documentation et trouver des informations sur
            l'installation, l'automatisation, les fonctionnalités avancées, et
            bien plus encore.
          </p>
        </section>

        <!-- Table des Matières -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Table des Matières
          </h2>
          <ul class="list-disc list-inside text-gray-700 space-y-4">
            <li>
              <a href="#installation" class="text-blue-600 underline"
                >Installation de France-Nuage</a
              >
            </li>
            <li>
              <a href="#configuration-initiale" class="text-blue-600 underline"
                >Configuration Initiale</a
              >
            </li>
            <li>
              <a
                href="#deploiement-applications"
                class="text-blue-600 underline"
                >Déploiement des Applications</a
              >
            </li>
            <li>
              <a href="#gestion-des-ressources" class="text-blue-600 underline"
                >Gestion des Ressources</a
              >
            </li>
            <li>
              <a href="#securite-et-sauvegarde" class="text-blue-600 underline"
                >Sécurité et Sauvegarde</a
              >
            </li>
            <li>
              <a href="#support-et-aide" class="text-blue-600 underline"
                >Support et Aide</a
              >
            </li>
          </ul>
        </section>

        <!-- Installation -->
        <section id="installation" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Installation de France-Nuage
          </h2>
          <p class="text-gray-700 mb-4">
            Pour commencer avec France-Nuage, vous devrez installer notre agent
            sur vos serveurs. L'agent est disponible pour plusieurs
            distributions Linux et peut être installé en suivant les étapes
            ci-dessous :
          </p>
          <ol class="list-decimal list-inside text-gray-700 space-y-3">
            <li>
              Téléchargez l'agent France-Nuage depuis notre
              <a href="#" class="text-blue-600 underline"
                >page de téléchargement</a
              >.
            </li>
            <li>Exécutez le script d'installation avec les privilèges root.</li>
            <li>
              Suivez les instructions à l'écran pour configurer la connexion
              avec votre compte France-Nuage.
            </li>
          </ol>
        </section>

        <!-- Configuration Initiale -->
        <section id="configuration-initiale" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Configuration Initiale
          </h2>
          <p class="text-gray-700">
            Après avoir installé l'agent, vous devrez configurer vos serveurs
            pour qu'ils puissent être utilisés avec France-Nuage. Cela inclut la
            définition des rôles des serveurs, la mise en place des clés SSH, et
            la connexion à notre interface de gestion.
          </p>
        </section>

        <!-- Déploiement des Applications -->
        <section id="deploiement-applications" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Déploiement des Applications
          </h2>
          <p class="text-gray-700 mb-4">
            France-Nuage permet de déployer des applications directement depuis
            votre tableau de bord. Choisissez parmi une liste d'applications
            préconfigurées ou utilisez vos propres images Docker pour créer des
            déploiements sur mesure.
          </p>
          <p class="text-gray-700">
            Pour déployer une application, rendez-vous dans la section
            "Applications" de votre tableau de bord, choisissez l'application
            souhaitée, puis cliquez sur "Déployer". Vous pouvez ensuite
            personnaliser les paramètres de votre déploiement selon vos besoins.
          </p>
        </section>

        <!-- Gestion des Ressources -->
        <section id="gestion-des-ressources" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Gestion des Ressources
          </h2>
          <p class="text-gray-700">
            Gérez facilement les ressources de votre infrastructure, incluant la
            CPU, la RAM, et le stockage. Utilisez notre interface pour ajuster
            la capacité de vos serveurs, configurer des règles d'auto-scaling,
            et surveiller l'utilisation en temps réel.
          </p>
        </section>

        <!-- Sécurité et Sauvegarde -->
        <section id="securite-et-sauvegarde" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Sécurité et Sauvegarde
          </h2>
          <p class="text-gray-700 mb-4">
            France-Nuage intègre des fonctionnalités avancées pour assurer la
            sécurité de vos données. Toutes les communications sont chiffrées,
            et des sauvegardes automatiques peuvent être configurées pour
            garantir que vos données sont toujours sécurisées.
          </p>
          <p class="text-gray-700">
            Vous pouvez configurer la fréquence des sauvegardes, choisir entre
            différents niveaux de redondance, et restaurer vos données à tout
            moment via votre tableau de bord.
          </p>
        </section>

        <!-- Support et Aide -->
        <section id="support-et-aide" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Support et Aide
          </h2>
          <p class="text-gray-700">
            Notre équipe est là pour vous aider à chaque étape. Si vous avez des
            questions ou des problèmes, n'hésitez pas à consulter notre FAQ, à
            lire nos guides détaillés, ou à contacter notre support technique.
          </p>
          <p class="text-gray-700 mt-4">
            Email :
            <a
              href="mailto:contact@france-nuage.fr"
              class="text-blue-600 underline"
              >contact@france-nuage.fr</a
            >
          </p>
        </section>
      </div>
    </div>
  </nuxt-layout>
</template>

<script setup lang="ts"></script>

<style scoped></style>
