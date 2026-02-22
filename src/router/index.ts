import { createRouter, createWebHistory } from 'vue-router'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
    },
    {
      path: '/profiles',
      name: 'profiles',
      component: () => import('@/views/ProfilesView.vue'),
    },
    {
      path: '/profiles/create',
      name: 'profile-create',
      component: () => import('@/views/ProfileCreateView.vue'),
    },
    {
      path: '/profiles/:id/edit',
      name: 'profile-edit',
      component: () => import('@/views/ProfileEditView.vue'),
      props: true,
    },
    {
      path: '/snapshots',
      name: 'snapshots',
      component: () => import('@/views/SnapshotsView.vue'),
    },
    {
      path: '/snapshots/create',
      name: 'snapshot-create',
      component: () => import('@/views/SnapshotCreateView.vue'),
    },
    {
      path: '/storage',
      name: 'storage',
      component: () => import('@/views/StorageView.vue'),
    },
    {
      path: '/settings',
      name: 'settings',
      component: () => import('@/views/SettingsView.vue'),
    },
    {
      path: '/guide',
      name: 'guide',
      component: () => import('@/views/GuideView.vue'),
    },
  ],
})

export default router
