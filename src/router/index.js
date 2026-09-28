import { createRouter, createWebHashHistory } from 'vue-router'
import HomeView from '../views/HomeView.vue'
import WorkspaceView from '../views/WorkspaceView.vue'
import { useProjectStore } from '../stores/project'

const routes = [
  { path: '/', component: HomeView },
  {
    path: '/workspace',
    component: WorkspaceView,
    beforeEnter: () => {
      const project = useProjectStore()
      if (!project.isOpen) return '/'
    },
  },
]

export default createRouter({
  history: createWebHashHistory(),
  routes,
})
