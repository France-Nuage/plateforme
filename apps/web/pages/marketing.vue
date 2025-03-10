<template>
  <nuxt-layout>
    <div class="relative isolate px-6 pt-14 lg:px-8 bg-gray-50">
      <div class="mx-auto max-w-4xl pt-20 pb-16 sm:pt-28 lg:pt-32">
        <!-- Titre Principal -->
        <h1 class="text-3xl font-bold text-gray-900 mb-8">
          Découvrez France-Nuage : Votre Cloud Souverain
        </h1>

        <!-- Pourquoi Choisir France-Nuage -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Pourquoi Choisir France-Nuage ?
          </h2>
          <p class="text-gray-700 mb-4">
            Reprenez le contrôle de vos données et de votre infrastructure avec
            <strong>France-Nuage</strong>. Nous proposons une solution cloud
            conçue pour garantir la <strong>souveraineté des données</strong>,
            la <strong>transparence des coûts</strong>, et une
            <strong>flexibilité totale</strong>. Contrairement aux fournisseurs
            traditionnels, France-Nuage vous offre la possibilité de choisir
            entre une infrastructure hébergée 100 % localement ou une gestion
            sur vos propres serveurs.
          </p>
          <p class="text-gray-700">
            Bénéficiez des avantages du cloud sans compromis :
            <strong>sécurité</strong>, <strong>scalabilité</strong>, et
            <strong>simplicité</strong>. Tout cela, sans renoncer au contrôle
            total de vos informations et sans les coûts opaques des grands
            fournisseurs.
          </p>
        </section>

        <!-- Comparaison avec les Cloud Providers Traditionnels -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Avec France-Nuage vs Les Cloud Providers Traditionnels
          </h2>
          <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
            <!-- Colonne Cloud Providers Traditionnels -->
            <div class="bg-white p-6 rounded-lg shadow-md">
              <h3 class="text-xl font-semibold text-gray-900 mb-4">
                Chez les cloud providers
              </h3>
              <ul class="list-none text-gray-700 space-y-3">
                <li>
                  <span class="text-red-500 font-bold">&times;</span>
                  Enfermement dans des solutions propriétaires
                </li>
                <li>
                  <span class="text-red-500 font-bold">&times;</span> Hébergé à
                  l'étranger, soumis à des lois non-européennes
                </li>
                <li>
                  <span class="text-red-500 font-bold">&times;</span>
                  Infrastructure imposée sans flexibilité
                </li>
                <li>
                  <span class="text-red-500 font-bold">&times;</span> Impossible
                  de changer de fournisseur sans gros travaux
                </li>
                <li>
                  <span class="text-red-500 font-bold">&times;</span> Interface
                  très complexe
                </li>
              </ul>
            </div>
            <!-- Colonne France-Nuage -->
            <div class="bg-blue-600 p-6 rounded-lg text-white">
              <h3 class="text-xl font-semibold text-white mb-4">
                Avec France-Nuage
              </h3>
              <ul class="list-none space-y-3">
                <li>
                  <span class="font-bold">✔</span> Quittez-nous à tout moment
                  et simplement
                </li>
                <li>
                  <span class="font-bold">✔</span> Mises à jour de sécurité
                  sans interruption
                </li>
                <li>
                  <span class="font-bold">✔</span> Écosystème open-source
                  unifié
                </li>
                <li>
                  <span class="font-bold">✔</span> Applications déjà
                  interconnectées
                </li>
                <li>
                  <span class="font-bold">✔</span> Choisissez entre vos propres
                  serveurs et ceux de France-Nuage
                </li>
                <li>
                  <span class="font-bold">✔</span> Contrôle complet sur vos
                  machines et vos données
                </li>
                <li><span class="font-bold">✔</span> Interface intuitive</li>
                <li>
                  <span class="font-bold">✔</span> Le tout pour un coût mesuré
                  et maîtrisé
                </li>
              </ul>
            </div>
          </div>
        </section>

        <!-- Plans et Tarifs -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Plans et Tarifs
          </h2>
          <p class="text-gray-700 mb-4">
            France-Nuage propose des offres adaptées à chaque besoin, que vous
            soyez une startup ou une grande entreprise. Choisissez le plan qui
            correspond à vos besoins et bénéficiez de tarifs compétitifs, sans
            mauvaises surprises.
          </p>
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <!-- Plan Autonome -->
            <div
              class="bg-white p-6 rounded-lg shadow-md flex flex-col justify-between"
            >
              <div>
                <h3 class="text-xl font-semibold text-gray-900 mb-4">
                  Autonome
                </h3>
                <p class="text-3xl font-bold text-gray-900 mb-4">15€</p>
                <ul class="list-none text-gray-700 space-y-3 mb-6">
                  <li>✔ Écosystèmes Open-Source</li>
                  <li>✔ Services interconnectés</li>
                  <li>✔ Supervision et alerting</li>
                  <li>✔ Chiffrement de bout en bout</li>
                  <li>✔ Uniquement vos ressources</li>
                </ul>
              </div>
              <a
                href="/contact"
                class="mt-auto w-full bg-blue-600 text-white py-2 rounded-lg text-center hover:bg-blue-700 transition"
              >
                Commencer dès maintenant
              </a>
            </div>

            <!-- Plan Startup -->
            <div
              class="bg-blue-600 p-6 rounded-lg text-white flex flex-col justify-between"
            >
              <div>
                <h3 class="text-xl font-semibold mb-4">Startup</h3>
                <p class="text-3xl font-bold mb-4">45€</p>
                <p class="mb-4">* le plan autonome plus :</p>
                <ul class="list-none space-y-3 mb-6">
                  <li>✔ 98% SLA</li>
                  <li>✔ Support sous 24h par mail</li>
                  <li>✔ Conformité RGPD</li>
                  <li>✔ Sauvegarde chez France-Nuage</li>
                  <li>✔ Failover avec France-Nuage</li>
                </ul>
              </div>
              <a
                href="/contact"
                class="mt-auto w-full bg-white text-blue-600 py-2 rounded-lg text-center hover:bg-gray-200 transition"
              >
                Commencer dès maintenant
              </a>
            </div>

            <!-- Plan Entreprise -->
            <div
              class="bg-white p-6 rounded-lg shadow-md flex flex-col justify-between"
            >
              <div>
                <h3 class="text-xl font-semibold text-gray-900 mb-4">
                  Entreprise
                </h3>
                <p class="text-3xl font-bold text-gray-900 mb-4">299€</p>
                <p class="mb-4">* le plan startup plus :</p>
                <ul class="list-none text-gray-700 space-y-3 mb-6">
                  <li>✔ 99,9% SLA</li>
                  <li>✔ Support dédié 24/7 avec réponse sous 4h</li>
                  <li>✔ Migrations sans interruptions</li>
                  <li>✔ Conformité RGPD, ISO 27001</li>
                  <li>✔ Sauvegarde multi-site</li>
                  <li>✔ Répartition de charge</li>
                </ul>
              </div>
              <a
                href="/contact"
                class="mt-auto w-full bg-blue-600 text-white py-2 rounded-lg text-center hover:bg-blue-700 transition"
              >
                Commencer dès maintenant
              </a>
            </div>
          </div>
        </section>

        <!-- Contactez-Nous -->
        <section class="mb-10">
          <h2 class="text-2xl font-semibold text-gray-800 mb-4">
            Contactez-Nous
          </h2>
          <p class="text-gray-700">
            Prêt à reprendre le contrôle de votre infrastructure cloud ?
            Contactez-nous dès aujourd'hui pour en savoir plus sur nos solutions
            et obtenir une démonstration gratuite.
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
