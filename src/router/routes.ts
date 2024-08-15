import type {RouteRecordRaw} from 'vue-router'


const routes: Array<RouteRecordRaw> = [
    {
        path: '/main',
        name: 'main',
        alias: '/',
        component: () => import('@view/MixTex.vue'),
    },
    {
        path: '/screenshot',
        name: 'screenshot',
        component: () => import('@view/Screenshot.vue'),
    },
    {
        path: '/:pathMatch(.*)*\'',
        name: 'Not Found',
        component: () => import('@view/NotFound.vue'),
    },
]


export default routes