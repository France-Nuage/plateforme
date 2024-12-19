<template>
  <div>
    <TransitionRoot as="template" :show="sidebarOpen">
      <Dialog class="relative z-50 lg:hidden" @close="sidebarOpen = false">
        <TransitionChild as="template" enter="transition-opacity ease-linear duration-300" enter-from="opacity-0" enter-to="opacity-100" leave="transition-opacity ease-linear duration-300" leave-from="opacity-100" leave-to="opacity-0">
          <div class="fixed inset-0 bg-gray-900/80" />
        </TransitionChild>

        <div class="fixed inset-0 flex">
          <TransitionChild as="template" enter="transition ease-in-out duration-300 transform" enter-from="-translate-x-full" enter-to="translate-x-0" leave="transition ease-in-out duration-300 transform" leave-from="translate-x-0" leave-to="-translate-x-full">
            <DialogPanel class="relative mr-16 flex w-full max-w-xs flex-1">
              <TransitionChild as="template" enter="ease-in-out duration-300" enter-from="opacity-0" enter-to="opacity-100" leave="ease-in-out duration-300" leave-from="opacity-100" leave-to="opacity-0">
                <div class="absolute left-full top-0 flex w-16 justify-center pt-5">
                  <button type="button" class="-m-2.5 p-2.5" @click="sidebarOpen = false">
                    <span class="sr-only">Close sidebar</span>
                    <XMarkIcon class="size-6 text-white" aria-hidden="true" />
                  </button>
                </div>
              </TransitionChild>
              <!-- Sidebar component, swap this element with another sidebar if you like -->
              <div class="flex grow flex-col gap-y-5 overflow-y-auto bg-gray-900 px-6 pb-4 ring-1 ring-white/10">
                <div class="flex h-16 shrink-0 items-center">
                  <c-logo />
                </div>
                <nav class="flex flex-1 flex-col">
                  <ul role="list" class="flex flex-1 flex-col gap-y-7">
                    <li>
                      <ul role="list" class="-mx-2 space-y-1">
                        <li v-for="item in navigation" :key="item.name">
                          <a :href="item.href" :class="[item.current ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800 hover:text-white', 'group flex gap-x-3 rounded-md p-2 text-sm/6 font-semibold']">
                            <component :is="item.icon" class="size-6 shrink-0" aria-hidden="true" />
                            {{ item.name }}
                          </a>
                        </li>
                      </ul>
                    </li>
                    <li>
                      <div class="text-xs/6 font-semibold text-gray-400">Your teams</div>
                      <ul role="list" class="-mx-2 mt-2 space-y-1">
                        <li v-for="team in teams" :key="team.name">
                          <a :href="team.href" :class="[team.current ? 'bg-gray-800 text-white' : 'text-gray-400 hover:bg-gray-800 hover:text-white', 'group flex gap-x-3 rounded-md p-2 text-sm/6 font-semibold']">
                            <span class="flex size-6 shrink-0 items-center justify-center rounded-lg border border-gray-700 bg-gray-800 text-[0.625rem] font-medium text-gray-400 group-hover:text-white">{{ team.initial }}</span>
                            <span class="truncate">{{ team.name }}</span>
                          </a>
                        </li>
                      </ul>
                    </li>
                    <li class="mt-auto">
                      <a href="#" class="group -mx-2 flex gap-x-3 rounded-md p-2 text-sm/6 font-semibold text-gray-400 hover:bg-gray-800 hover:text-white">
                        <Cog6ToothIcon class="size-6 shrink-0" aria-hidden="true" />
                        Settings
                      </a>
                    </li>
                  </ul>
                </nav>
              </div>
            </DialogPanel>
          </TransitionChild>
        </div>
      </Dialog>
    </TransitionRoot>

    <c-sidebar>

      <c-sidebar-navigation-group>
        <button class="text-white text-sm text-start flex items-center gap-2" @click="$router.push('/')">
          <Icon :name="`solar:backspace-bold-duotone`" size="1.3rem" style="color: #c1c1c1" />
          <span>Retour</span>
        </button>
      </c-sidebar-navigation-group>

      <c-sidebar-navigation-group title="Profile">
        <c-sidebar-navigation-item title="Préférences" icon="user-circle-bold-duotone" to="/profile/preferences" :active="route.path === '/profile/preferences'" />
        <c-sidebar-navigation-item title="Token personnel" icon="key-bold-duotone" to="/profile/access-token" :active="route.path === '/profile/access-token'" />
        <c-sidebar-navigation-item title="Sécurité" icon="lock-keyhole-bold-duotone" to="/profile/security" :active="route.path === '/profile/security'" />
      </c-sidebar-navigation-group>

      <c-sidebar-navigation-group title="Documentations">
        <c-sidebar-navigation-item title="Guides" icon="square-bottom-up-line-duotone" to="/instances" :active="isActive('/documentation/guides')" />
        <c-sidebar-navigation-item title="Api référence" icon="square-bottom-up-line-duotone" to="/services" :active="isActive('/documentation/api')" />
      </c-sidebar-navigation-group>

    </c-sidebar>

    <div class="lg:pl-72 bg-gray-50 dark:bg-gray-900 min-h-screen">
      <c-header />
      <main class="py-10">
        <div class="px-4 sm:px-6 lg:px-8">
          <slot />
        </div>
      </main>
    </div>
    <c-toast-list />
  </div>
</template>

<script lang="ts" setup>
import {
  Dialog,
  DialogPanel,
  TransitionChild,
  TransitionRoot,
} from '@headlessui/vue'
import {
  Cog6ToothIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
import CLogo from "~/components/CLogo.vue";
import CHeader from "~/components/header/CHeader.vue";
import { ref } from "vue";
import CSidebar from "~/components/sidebar/CSidebar.vue";
import CSidebarNavigationItem from "~/components/sidebar/CSidebarNavigationItem.vue";
import CSidebarNavigationGroup from "~/components/sidebar/CSidebarNavigationGroup.vue";
import CToastList from "~/components/toast/CToastList.vue";

const sidebarOpen = ref(false)
const route = useRoute()
const isActive = (term: string) => {
  return route.path.startsWith('/' + term)
}
</script>