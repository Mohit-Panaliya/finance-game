<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonPage, IonContent } from '@ionic/vue'
import GameInput from '@/components/game/GameInput.vue'
import GameButton from '@/components/game/GameButton.vue'
import { useAuthStore } from '@/stores/authStore'

const router = useRouter()
const auth = useAuthStore()

const name = ref('')
const email = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')

async function submit() {
  if (!name.value || !email.value || !password.value) {
    error.value = 'All fields required, squire!'
    return
  }
  if (password.value.length < 6) {
    error.value = 'Secret word must be 6+ characters'
    return
  }
  busy.value = true
  error.value = ''
  try {
    await auth.register(email.value, name.value, password.value)
    await router.replace('/')
  } catch {
    error.value = auth.error ?? 'Registration failed'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ion-page>
    <ion-content :fullscreen="true" class="reg-bg">
      <div class="wrap">
        <div class="banner-pole" aria-hidden="true" />
        <div class="reg-panel panel-stone noise bounce-in">
          <div class="crest">🛡️</div>
          <h1 class="carved carved-gold title">RAISE YOUR BANNER</h1>
          <p class="subtitle carved">Found a new fortress</p>

          <form class="form ff-col" @submit.prevent="submit">
            <GameInput v-model="name" label="Champion Name" placeholder="e.g. Aldric the Bold" />
            <GameInput v-model="email" type="email" label="Raven Address" placeholder="you@keep.com" />
            <GameInput v-model="password" type="password" label="Secret Word" placeholder="6+ characters" />
            <p v-if="error" class="error carved carved-sm">{{ error }}</p>
            <GameButton type="submit" variant="green" size="lg" block :disabled="busy" sparkle>
              {{ busy ? 'Building…' : 'BUILD FORTRESS' }}
            </GameButton>
          </form>

          <div class="links">
            <button class="link" type="button" @click="router.push('/login')">
              Already sworn in? <span class="link-gold">Enter gate →</span>
            </button>
          </div>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.reg-bg {
  --background: radial-gradient(circle at 50% 10%, #3e2712 0%, #241405 50%, #0d0700 100%);
  --color: var(--ff-parchment);
}
.wrap {
  position: relative;
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 18px calc(40px + env(safe-area-inset-bottom));
}
.banner-pole {
  position: absolute;
  top: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 8px;
  height: 90px;
  background: linear-gradient(180deg, #6b431f, #3a220d);
  border: 2px solid #1a0f00;
}
.reg-panel {
  position: relative;
  width: min(400px, 100%);
  padding: 26px 20px 22px;
  text-align: center;
}
.crest {
  font-size: 44px;
  margin-top: -52px;
  margin-bottom: 6px;
  filter: drop-shadow(0 4px 0 rgba(0, 0, 0, 0.6));
  animation: bob 2.5s ease-in-out infinite;
}
.title {
  margin: 0;
  font-size: 25px;
  letter-spacing: 0.05em;
}
.subtitle {
  margin: 4px 0 18px;
  font-size: 13px;
  opacity: 0.75;
  letter-spacing: 0.16em;
  text-transform: uppercase;
}
.form {
  gap: 14px;
  text-align: left;
}
.error {
  color: #ff9d94;
  text-align: center;
  margin: 0;
}
.links {
  margin-top: 16px;
}
.link {
  background: none;
  border: none;
  color: var(--ff-parchment);
  font-family: var(--ff-font-body);
  font-size: 14px;
  cursor: pointer;
  opacity: 0.85;
}
.link-gold {
  color: var(--ff-gold);
  font-weight: 800;
}
</style>
