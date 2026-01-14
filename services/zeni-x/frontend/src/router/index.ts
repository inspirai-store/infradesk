import { createRouter, createWebHistory } from 'vue-router'
import type { RouteRecordRaw } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
  },
  {
    path: '/connections',
    name: 'Connections',
    component: () => import('@/views/connections/ConnectionsView.vue'),
  },
  {
    path: '/port-forward',
    name: 'PortForward',
    component: () => import('@/views/PortForwardView.vue'),
  },
  {
    path: '/mysql',
    name: 'MySQL',
    component: () => import('@/views/mysql/MySQLView.vue'),
    children: [
      {
        path: '',
        name: 'MySQLDatabases',
        component: () => import('@/views/mysql/DatabaseList.vue'),
      },
      {
        path: ':database',
        name: 'MySQLTables',
        component: () => import('@/views/mysql/TableList.vue'),
      },
      {
        path: ':database/:table',
        name: 'MySQLTableData',
        component: () => import('@/views/mysql/TableData.vue'),
      },
      {
        path: ':database/objects',
        name: 'MySQLObjects',
        component: () => import('@/views/mysql/components/DatabaseObjectsView.vue'),
        props: true,
      },
      {
        path: 'server',
        name: 'MySQLServer',
        component: () => import('@/views/mysql/ServerMonitorView.vue'),
      },
      {
        path: 'query',
        name: 'MySQLQuery',
        component: () => import('@/views/mysql/QueryEditor.vue'),
      },
      {
        path: 'users',
        name: 'MySQLUsers',
        component: () => import('@/views/mysql/UsersView.vue'),
      },
    ],
  },
  {
    path: '/redis',
    name: 'Redis',
    component: () => import('@/views/redis/RedisView.vue'),
    children: [
      {
        path: '',
        name: 'RedisKeys',
        component: () => import('@/views/redis/KeyBrowser.vue'),
      },
      {
        path: 'key/:key',
        name: 'RedisKeyDetail',
        component: () => import('@/views/redis/KeyDetail.vue'),
      },
    ],
  },
  {
    path: '/k8s-resources',
    name: 'K8sResources',
    component: () => import('@/views/k8s/K8sResourcesView.vue'),
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router

