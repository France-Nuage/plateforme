import { createRouter, createWebHistory } from 'vue-router'

import ListPageIndex from '../views/compute/list-page.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/compute', name: 'compute-index', component: ListPageIndex },
  ],
})

export default router
