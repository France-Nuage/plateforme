import { createRouter, createWebHistory } from 'vue-router'

import ComputeIndex from '../views/compute/index.vue'
import ComputeId from '../views/compute/[id].vue'
import ComputeCreate from '../views/compute/create.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    { path: '/compute', name: 'compute-index', component: ComputeIndex },
    { path: '/compute/:id', name: 'compute-id', component: ComputeId, props: true },
    { path: '/compute/create', name: 'compute-create', component: ComputeCreate },
  ],
})

export default router
