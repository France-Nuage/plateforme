<template>
    <div class="bg-white py-24 sm:py-32">
        <div class="mx-auto grid max-w-7xl gap-20 px-6 lg:px-8 xl:grid-cols-3">
            <div class="max-w-xl bg-primary rounded-3xl p-8 xl:p-10">
                <h2 class="text-pretty text-3xl font-semibold tracking-tight text-white sm:text-4xl">Les visages de France-Nuage</h2>
                <p class="mt-4 font-tight text-lg/7 text-white">Fondé par une équipe d'indépendants, France-Nuage regroupe des alumnis fatigués du modèle de cloud traditionnel, voulant proposer une alternative où le client redeviens maitre de son destin. </p>
            </div>
            <ul role="list" class="grid sm:grid-cols-2 gap-y-12 xl:col-span-2 h-full items-center">
            <li>
                <div class="flex items-center gap-x-6">
                    <img class="size-20 rounded-full" :src="`/images/profil_seb.jpeg`" alt="">
                    <div>
                        <h3 class="text-base/7 font-semibold tracking-tight text-gray-900">Sébastien BERTHOME</h3>
                        <p class="text-sm/6 font-semibold" style="color: #284ada;">Ex. Opendatasoft / </p>
                    </div>
                </div>
            </li>
            <li>
                <div class="flex items-center gap-x-6">
                    <img class="size-20 rounded-full" :src="`/images/profil_alex.jpeg`" alt="">
                    <div>
                        <h3 class="text-base/7 font-semibold tracking-tight text-gray-900">Alexandre CAILLER</h3>
                        <p class="text-sm/6 font-semibold" style="color: #284ada;">Ex. Ouest-France</p>
                    </div>
                </div>
            </li>
            <li>
                <div class="flex items-center gap-x-6">
                    <img class="size-20 rounded-full" :src="`/images/profil_kev.jpeg`" alt="">
                    <div>
                        <h3 class="text-base/7 font-semibold tracking-tight text-gray-900">Kévin GILLET</h3>
                        <p class="text-sm/6 font-semibold" style="color: #284ada;">Ex. Système-U</p>
                    </div>
                </div>
            </li>
            <li>
                <div class="flex items-center gap-x-6">
                    <img class="size-20 rounded-full" :src="`/images/profil_fg.jpeg`" alt="">
                    <div>
                        <h3 class="text-base/7 font-semibold tracking-tight text-gray-900">François-Guillaume RIBREAU</h3>
                        <p class="text-sm/6 font-semibold" style="color: #284ada;">Ex. Ouest-France / CloudIAM / Redsmin</p>
                    </div>
                </div>
            </li>
            </ul>
        </div>
    </div>
</template>