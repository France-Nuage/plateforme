<template>
  <nuxt-layout>
    <div class="relative isolate px-6 pt-14 lg:px-8 bg-gray-50">
      <div class="mx-auto max-w-4xl pt-20 pb-16 sm:pt-28 lg:pt-32">
        <!-- Titre Principal -->
        <h1 class="text-3xl font-bold text-gray-900 mb-8">
          Soumettre un Ticket de Support
        </h1>

        <!-- Introduction -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Nous n'aimons pas que vous ayez besoin de nous contacter
          </h2>
          <p class="text-gray-700 mb-4">
            Chez France-Nuage, nous croyons que notre service doit fonctionner
            sans que vous ayez à vous soucier des détails. Si vous avez besoin
            de créer un ticket, c'est que nous n'avons pas fait notre travail en
            tant qu'infogéreur de nouvelle génération. Notre objectif est de
            créer une expérience fluide et sans faille, où tout est déjà pris en
            charge pour vous, et où VOUS avez le contrôle.
          </p>
          <p class="text-gray-700 mb-4">
            Alors, quand vous devez soumettre un ticket, nous le prenons
            personnellement. C'est un signe que nous n'avons pas bien fait notre
            travail. Chaque ticket est pour nous une opportunité de comprendre
            ce qui n'a pas fonctionné et de productiviser votre problème au sein
            de notre produit, afin de vous rendre votre autonomie et votre
            pouvoir d'action.
          </p>
          <p class="text-gray-700">
            Nous sommes désolés de ne pas avoir répondu à vos attentes cette
            fois-ci, mais nous sommes déterminés à faire mieux. Merci de prendre
            le temps de nous informer du problème.
          </p>
        </section>

        <!-- Formulaire de Soumission de Ticket -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Soumettre un Ticket
          </h2>
          <form
            action="/submit-ticket"
            method="post"
            class="bg-white p-6 rounded-lg shadow-md"
          >
            <div class="mb-4">
              <label for="name" class="block text-sm font-medium text-gray-700"
                >Nom</label
              >
              <input
                type="text"
                id="name"
                name="name"
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div class="mb-4">
              <label for="email" class="block text-sm font-medium text-gray-700"
                >Adresse Email</label
              >
              <input
                type="email"
                id="email"
                name="email"
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              />
            </div>
            <div class="mb-4">
              <label for="issue" class="block text-sm font-medium text-gray-700"
                >Description du Problème</label
              >
              <textarea
                id="issue"
                name="issue"
                rows="4"
                required
                class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500"
              ></textarea>
            </div>
            <button
              type="submit"
              class="w-full bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700"
            >
              Soumettre
            </button>
          </form>
        </section>

        <!-- Engagement à S'améliorer -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Notre Engagement à Nous Améliorer
          </h2>
          <p class="text-gray-700">
            Chaque ticket soumis est examiné attentivement par notre équipe.
            Nous analysons chaque incident pour identifier les points à
            améliorer et éviter que cela ne se reproduise. Notre objectif est de
            rendre France-Nuage meilleur chaque jour pour vous offrir une
            expérience toujours plus fiable et simple.
          </p>
          <p class="text-gray-700 mt-4">
            Merci de votre patience et de votre compréhension. Ensemble, nous
            ferons en sorte que l'utilisation de France-Nuage soit toujours sans
            effort.
          </p>
        </section>

        <!-- Contactez-Nous Directement -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Contactez-Nous Directement
          </h2>
          <p class="text-gray-700">
            Vous préférez nous contacter directement ou avez besoin d'une
            assistance immédiate ? Vous pouvez nous joindre à :
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
