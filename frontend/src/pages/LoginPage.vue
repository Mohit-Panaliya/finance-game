<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { IonPage, IonContent } from '@ionic/vue'
import GameInput from '@/components/game/GameInput.vue'
import GameButton from '@/components/game/GameButton.vue'
import { useAuthStore } from '@/stores/authStore'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')
const gateReady = ref(false)

onMounted(() => {
  requestAnimationFrame(() => {
    gateReady.value = true
  })
})

async function submit() {
  if (!username.value || !password.value) {
    error.value = 'Fill both fields, recruit!'
    return
  }
  busy.value = true
  error.value = ''
  try {
    await auth.login(username.value, password.value)
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/'
    await router.replace(redirect)
  } catch {
    error.value = auth.error ?? 'Login failed'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ion-page>
    <ion-content :fullscreen="true" class="login-bg">
      <div class="gate">
        <div class="gate-wall" aria-hidden="true" />
        <div class="gate-arch" :class="{ ready: gateReady }" aria-hidden="true">
          <div class="gate-door" />
        </div>

        <div class="torch torch-l" aria-hidden="true">
          <span class="torch-flame" />
          <span class="torch-glow" />
        </div>
        <div class="torch torch-r" aria-hidden="true">
          <span class="torch-flame" />
          <span class="torch-glow" />
        </div>

        <div class="login-panel panel-stone noise bounce-in">
          <div class="crest">⚔️</div>
          <h1 class="carved carved-gold title">FINANCE FORGE</h1>
          <p class="subtitle carved">Enter the fortress</p>

          <form class="form ff-col" @submit.prevent="submit">
            <GameInput
              v-model="username"
              label="Champion"
              placeholder="Username or email"
              autocomplete="username"
            />
            <GameInput
              v-model="password"
              type="password"
              label="Secret Word"
              placeholder="Password"
              autocomplete="current-password"
            />
            <p v-if="error" class="error carved carved-sm">{{ error }}</p>
            <GameButton type="submit" variant="gold" size="lg" block :disabled="busy" sparkle>
              {{ busy ? 'Opening gates…' : 'ENTER FORTRESS' }}
            </GameButton>
          </form>

          <div class="links">
            <button class="link" type="button" @click="router.push('/register')">
              New here? <span class="link-gold">Raise a banner →</span>
            </button>
          </div>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.login-bg {
  --background: radial-gradient(circle at 50% 0%, #4a2c12 0%, #241405 45%, #0d0700 100%);
  --color: var(--ff-parchment);
}
.gate {
  position: relative;
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 34px 18px calc(34px + env(safe-area-inset-bottom));
  overflow: hidden;
}
.gate-wall {
  position: absolute;
  inset: 0;
  background:
    repeating-linear-gradient(0deg, rgba(0, 0, 0, 0.22) 0 2px, transparent 2px 46px),
    repeating-linear-gradient(90deg, rgba(0, 0, 0, 0.22) 0 2px, transparent 2px 74px),
    linear-gradient(180deg, #3e2712 0%, #2a1a0a 60%, #1a0f00 100%);
  opacity: 0.55;
  pointer-events: none;
}
.gate-arch {
  position: absolute;
  top: 4%;
  width: min(320px, 80vw);
  height: min(320px, 44vh);
  border: 10px solid #4a3218;
  border-bottom: none;
  border-radius: 50% 50% 0 0 / 70% 70% 0 0;
  box-shadow:
    inset 0 0 40px rgba(0, 0, 0, 0.7),
    0 0 0 6px #1a0f00;
  opacity: 0.75;
  transform: scaleY(0.6);
  transform-origin: top;
  transition: transform 1s var(--ff-bounce);
  pointer-events: none;
}
.gate-arch.ready {
  transform: scaleY(1);
}
.gate-door {
  position: absolute;
  inset: 12px;
  border-radius: 46% 46% 0 0 / 66% 66% 0 0;
  background:
    repeating-linear-gradient(90deg, rgba(0, 0, 0, 0.35) 0 3px, transparent 3px 34px),
    linear-gradient(180deg, #5c3817, #2c1808);
  animation: gate-sway 6s ease-in-out infinite;
  transform-origin: top center;
}
.torch {
  position: absolute;
  top: 26%;
  width: 26px;
  height: 130px;
  z-index: 1;
  pointer-events: none;
}
.torch-l { left: 8%; }
.torch-r { right: 8%; }
.torch::after {
  content: '';
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 12px;
  height: 90px;
  background: linear-gradient(180deg, #6b431f, #3a220d);
  border: 3px solid #1a0f00;
  border-radius: 4px;
}
.torch-flame {
  position: absolute;
  top: -6px;
  left: 50%;
  transform: translateX(-50%);
  width: 30px;
  height: 44px;
  background: radial-gradient(circle at 50% 70%, #fff6d0 0%, #f5c542 35%, #ff7a1a 70%, rgba(231, 76, 60, 0.2) 100%);
  border-radius: 50% 50% 42% 42%;
  filter: blur(1px);
  animation: flame 0.55s ease-in-out infinite;
  z-index: 2;
  box-shadow: 0 0 34px 12px rgba(245, 160, 40, 0.45);
}
.torch-glow {
  position: absolute;
  top: -40px;
  left: 50%;
  transform: translateX(-50%);
  width: 120px;
  height: 120px;
  border-radius: 50%;
  background: radial-gradient(circle, rgba(245, 197, 66, 0.35), transparent 70%);
  animation: flame 0.9s ease-in-out infinite;
}
.login-panel {
  position: relative;
  z-index: 2;
  width: min(400px, 100%);
  padding: 26px 20px 22px;
  text-align: center;
}
.crest {
  font-size: 46px;
  margin-top: -54px;
  margin-bottom: 6px;
  filter: drop-shadow(0 4px 0 rgba(0, 0, 0, 0.6));
  animation: bob 2.4s ease-in-out infinite;
}
.title {
  margin: 0;
  font-size: 30px;
  letter-spacing: 0.06em;
}
.subtitle {
  margin: 4px 0 18px;
  font-size: 14px;
  opacity: 0.75;
  letter-spacing: 0.18em;
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
  text-shadow: 0 1px 0 #000;
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
  text-shadow: 0 1px 0 #5c3c00;
}
</style>
