<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { IonContent, IonPage } from '@ionic/vue'
import { useAuthStore } from '@/stores/authStore'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

const router = useRouter()
const route = useRoute()
const auth = useAuthStore()

const username = ref('')
const password = ref('')
const usernameError = ref('')
const passwordError = ref('')
const showPassword = ref(false)
const busy = ref(false)
const error = ref('')

function setUsername(value: string | number) {
  username.value = String(value)
  usernameError.value = ''
}

function setPassword(value: string | number) {
  password.value = String(value)
  passwordError.value = ''
}

async function submit() {
  usernameError.value = username.value.trim() ? '' : 'Enter your username or email.'
  passwordError.value = password.value ? '' : 'Enter your password.'
  if (usernameError.value || passwordError.value) return
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
          <h1 class="auth-title">Fintrack</h1>
          <p class="auth-sub">Sign in to your accounts</p>

          <form class="form-grid" @submit.prevent="submit">
            <AppInput
              :model-value="username"
              label="Username"
              placeholder="Username or email"
              autocomplete="username"
              :error="usernameError"
              @update:model-value="setUsername"
            />
            <div class="pw">
              <AppInput
                :model-value="password"
                :type="showPassword ? 'text' : 'password'"
                label="Password"
                placeholder="Your password"
                autocomplete="current-password"
                :error="passwordError"
                @update:model-value="setPassword"
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
  padding: var(--density-page-pad) var(--density-page-pad)
    calc(var(--density-page-pad) + 12px + env(safe-area-inset-bottom));
}
.auth-card {
  width: min(400px, 100%);
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: var(--radius-lg);
  padding: var(--density-hero-pad);
  box-shadow: var(--shadow-md);
  text-align: center;
}
.auth-sub {
  margin: 5px 0 var(--density-section-gap);
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
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
}
.auth-alt {
  margin: var(--density-section-gap) 0 0;
  font-size: 0.85rem;
  color: var(--text-muted);
}
.auth-link {
  color: var(--accent);
  font-weight: 600;
  text-decoration: none;
}
</style>