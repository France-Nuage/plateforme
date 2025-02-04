<template>
  <nuxt-layout>
    <div class="relative isolate px-6 pt-14 lg:px-8 bg-gray-50">
      <div class="mx-auto max-w-4xl pt-20 pb-16 sm:pt-28 lg:pt-32">
        <!-- Titre Principal -->
        <h1 class="text-3xl font-bold text-gray-900 mb-8">
          Guides France-Nuage
        </h1>

        <!-- Introduction -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Introduction
          </h2>
          <p class="text-gray-700 mb-4">
            Cette section regroupe une série de guides détaillés pour vous aider
            à tirer le meilleur parti de France-Nuage. Que vous débutiez avec le
            cloud ou que vous soyez un administrateur expérimenté, ces guides
            sont conçus pour répondre à vos questions, résoudre des problèmes
            courants, et vous accompagner pas à pas dans vos déploiements.
          </p>
          <p class="text-gray-700">
            Utilisez la table des matières ci-dessous pour explorer nos guides.
            Si vous avez des questions supplémentaires, notre équipe est là pour
            vous aider.
          </p>
        </section>

        <!-- Table des Matières -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Table des Matières
          </h2>
          <ul class="list-disc list-inside text-gray-700 space-y-4">
            <li>
              <a href="#guide-demarrage" class="text-blue-600 underline"
                >Guide de Démarrage</a
              >
            </li>
            <li>
              <a href="#gestion-serveurs" class="text-blue-600 underline"
                >Gestion des Serveurs et Machines Virtuelles</a
              >
            </li>
            <li>
              <a href="#deploiement-docker" class="text-blue-600 underline"
                >Déploiement avec Docker</a
              >
            </li>
            <li>
              <a href="#supervision-et-alerting" class="text-blue-600 underline"
                >Supervision et Alerting</a
              >
            </li>
            <li>
              <a href="#securite-cloud" class="text-blue-600 underline"
                >Sécurité dans le Cloud</a
              >
            </li>
            <li>
              <a
                href="#migration-vers-france-nuage"
                class="text-blue-600 underline"
                >Migration vers France-Nuage</a
              >
            </li>
          </ul>
        </section>

        <!-- Guide de Démarrage -->
        <section id="guide-demarrage" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Guide de Démarrage
          </h2>
          <p class="text-gray-700 mb-4">
            Ce guide est destiné aux nouveaux utilisateurs souhaitant démarrer
            rapidement avec France-Nuage. Vous apprendrez à créer un compte, à
            configurer vos premiers serveurs, et à déployer votre première
            application.
          </p>
          <ol class="list-decimal list-inside text-gray-700 space-y-3">
            <li>
              Créez un compte sur notre plateforme et vérifiez votre adresse
              email.
            </li>
            <li>
              Connectez-vous à l'interface et ajoutez vos premiers serveurs.
            </li>
            <li>
              Déployez une application à partir de notre catalogue pour tester
              la plateforme.
            </li>
          </ol>
        </section>

        <!-- Gestion des Serveurs et Machines Virtuelles -->
        <section id="gestion-serveurs" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Gestion des Serveurs et Machines Virtuelles
          </h2>
          <p class="text-gray-700 mb-4">
            Apprenez à gérer vos serveurs et machines virtuelles avec
            France-Nuage. Ce guide couvre la création de nouvelles machines
            virtuelles, la configuration des ressources (CPU, RAM, stockage), et
            la gestion de la haute disponibilité.
          </p>
          <p class="text-gray-700">
            Avec France-Nuage, vous pouvez facilement provisionner des serveurs
            pour répondre aux besoins croissants de votre entreprise, tout en
            gardant le contrôle sur chaque aspect de l'infrastructure.
          </p>
        </section>

        <!-- Déploiement avec Docker -->
        <section id="deploiement-docker" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Déploiement avec Docker
          </h2>
          <p class="text-gray-700 mb-4">
            Docker est un excellent outil pour déployer des applications de
            manière rapide et fiable. Ce guide vous montre comment utiliser
            Docker avec France-Nuage pour créer des environnements isolés,
            déployer vos propres conteneurs, et assurer la gestion continue de
            vos applications.
          </p>
          <p class="text-gray-700">
            Vous apprendrez à créer vos conteneurs, à les orchestrer, et à
            intégrer Docker à l'infrastructure existante pour un déploiement
            fluide et sécurisé.
          </p>
        </section>

        <!-- Supervision et Alerting -->
        <section id="supervision-et-alerting" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Supervision et Alerting
          </h2>
          <p class="text-gray-700">
            Ce guide vous explique comment superviser votre infrastructure et
            configurer des alertes personnalisées. France-Nuage offre des outils
            de surveillance avancés pour suivre les performances de vos serveurs
            et être immédiatement averti en cas de problème.
          </p>
        </section>

        <!-- Sécurité dans le Cloud -->
        <section id="securite-cloud" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Sécurité dans le Cloud
          </h2>
          <p class="text-gray-700">
            La sécurité est essentielle lorsqu'il s'agit de cloud computing.
            Dans ce guide, vous découvrirez les meilleures pratiques pour
            sécuriser vos données et applications. Nous couvrons le chiffrement,
            la gestion des accès, et les sauvegardes pour assurer la protection
            de vos données sensibles.
          </p>
        </section>

        <!-- Migration vers France-Nuage -->
        <section id="migration-vers-france-nuage" class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Migration vers France-Nuage
          </h2>
          <p class="text-gray-700 mb-4">
            Si vous envisagez de migrer vers France-Nuage depuis un autre
            fournisseur, ce guide est fait pour vous. Nous expliquons les étapes
            nécessaires pour transférer vos données, reconfigurer vos
            applications, et assurer une transition en douceur sans interruption
            de service.
          </p>
          <p class="text-gray-700">
            Notre équipe est là pour vous accompagner à chaque étape, afin que
            votre migration soit la plus fluide possible.
          </p>
        </section>

        <!-- Contactez-Nous -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Contactez-Nous
          </h2>
          <p class="text-gray-700">
            Si vous avez des questions sur un guide ou avez besoin de plus
            d'assistance, notre équipe est disponible pour vous aider.
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
