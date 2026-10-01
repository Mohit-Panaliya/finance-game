<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { walletOutline } from 'ionicons/icons'
import { useAuthStore } from '@/stores/authStore'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const showPassword = ref(false)
const busy = ref(false)
const error = ref('')

async function submit() {
  if (!username.value.trim() || !password.value) {
    error.value = 'Enter your username and password.'
    return
  }
  busy.value = true
  error.value = ''
  try {
    await auth.login(username.value.trim(), password.value)
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : '/dashboard'
    await router.replace(redirect)
  } catch {
    error.value = auth.error ?? 'Sign in failed'
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <ion-page>
    <ion-content :fullscreen="true" class="auth-bg">
      <div class="auth-wrap">
        <div class="auth-card">
          <div class="auth-mark"><ion-icon :icon="walletOutline" /></div>
          <h1 class="auth-title">Fintrack</h1>
          <p class="auth-sub">Sign in to your accounts</p>

          <form class="form-grid" @submit.prevent="submit">
            <AppInput
              v-model="username"
              label="Username"
              placeholder="Username or email"
              autocomplete="username"
            />
            <div class="pw">
              <AppInput
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                label="Password"
                placeholder="Your password"
                autocomplete="current-password"
              />
              <button class="pw-toggle" type="button" @click="showPassword = !showPassword">
                {{ showPassword ? 'Hide' : 'Show' }}
              </button>
            </div>

            <p v-if="error" class="form-error">{{ error }}</p>

            <AppButton type="submit" variant="primary" size="lg" block :disabled="busy">
              {{ busy ? 'Signing in…' : 'Sign in' }}
            </AppButton>
          </form>

          <p class="auth-alt">
            No account yet?
            <router-link class="auth-link" to="/register">Create one</router-link>
          </p>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.auth-bg {
  --background: var(--bg);
}
.auth-wrap {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 24px 18px calc(28px + env(safe-area-inset-bottom));
}
.auth-card {
  width: min(400px, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: 26px 20px 20px;
  box-shadow: var(--shadow-md);
  text-align: center;
}
.auth-mark {
  width: 52px;
  height: 52px;
  margin: 0 auto 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: var(--accent-soft);
  color: var(--accent);
  font-size: 26px;
}
.auth-title {
  margin: 0;
  font-size: 1.6rem;
  font-weight: 700;
  letter-spacing: -0.02em;
}
.auth-sub {
  margin: 5px 0 22px;
  font-size: 0.88rem;
  color: var(--text-muted);
}
.form-grid {
  text-align: left;
}
.pw {
  position: relative;
}
.pw-toggle {
  position: absolute;
  top: 30px;
  right: 10px;
  background: transparent;
  border: none;
  color: var(--text-muted);
  font-family: var(--font-body);
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}
.auth-alt {
  margin: 20px 0 0;
  font-size: 0.85rem;
  color: var(--text-muted);
}
.auth-link {
  color: var(--accent);
  font-weight: 600;
  text-decoration: none;
}
</style>