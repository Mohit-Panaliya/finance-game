import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { IonicVue } from '@ionic/vue'
import '@ionic/vue/css/core.css'
import '@ionic/vue/css/normalize.css'
import '@ionic/vue/css/structure.css'
import '@ionic/vue/css/typography.css'
import '@ionic/vue/css/padding.css'
import '@ionic/vue/css/float-elements.css'
import '@ionic/vue/css/text-alignment.css'
import '@ionic/vue/css/text-transformation.css'
// Ionic 8 ships no light palette; this app is dark-only by design.
import '@ionic/vue/css/palettes/dark.always.css'
import './theme/app.css'
// Style presets after app.css on purpose: equal-specificity preset rules have
// to win against the base component rules above.
import './theme/styles/index.css'

import App from './App.vue'
import router from './router'
import { watchPlatform } from './composables/usePlatform'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
// Android gets Material components, iOS keeps its native styling, and a
// desktop browser is told to render Material so the web build matches Windows.
const mode: 'ios' | 'md' = /android/i.test(navigator.userAgent)
  ? 'md'
  : /iphone|ipad|ipod/i.test(navigator.userAgent)
    ? 'ios'
    : 'md'

app.use(IonicVue, {
  mode,
  swipeBackEnabled: false,
  hardwareBackButton: true,
  backButtonText: ''
})
app.use(router)

watchPlatform()

router.isReady().then(() => {
  app.mount('#app')
})
