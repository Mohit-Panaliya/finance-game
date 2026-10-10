<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonPage } from '@ionic/vue'
import { useAuthStore } from '@/stores/authStore'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

const router = useRouter()
const auth = useAuthStore()

const name = ref('')
const email = ref('')
const password = ref('')
const nameError = ref('')
const emailError = ref('')
const passwordError = ref('')
const busy = ref(false)
const error = ref('')

function setName(value: string | number) {
  name.value = String(value)
  nameError.value = ''
}

function setEmail(value: string | number) {
  email.value = String(value)
  emailError.value = ''
}

function setPassword(value: string | number) {
  password.value = String(value)
  passwordError.value = ''
}

async function submit() {
  nameError.value = name.value.trim() ? '' : 'Enter your name.'
  const trimmedEmail = email.value.trim()
  emailError.value = !trimmedEmail
    ? 'Enter your email address.'
    : /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(trimmedEmail)
      ? ''
      : 'Use a format like you@example.com.'
  passwordError.value = !password.value
    ? 'Create a password.'
    : password.value.length < 6
      ? 'Use at least 6 characters.'
      : ''
  if (nameError.value || emailError.value || passwordError.value) return
  busy.value = true
  error.value = ''
  try {
    await auth.register(trimmedEmail, name.value.trim(), password.value)
    await router.replace('/dashboard')
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
      <div class="auth-wrap">
        <div class="auth-card">
          <h1 class="auth-title">Create your account</h1>
          <p class="auth-sub">Start tracking banks, deposits and investments</p>

          <form class="form-grid" @submit.prevent="submit">
            <AppInput
              :model-value="name"
              label="Name"
              placeholder="Your name"
              autocomplete="name"
              :error="nameError"
              @update:model-value="setName"
            />
            <AppInput
              :model-value="email"
              type="email"
              label="Email"
              placeholder="you@example.com"
              autocomplete="email"
              :error="emailError"
              @update:model-value="setEmail"
            />
            <AppInput
              :model-value="password"
              type="password"
              label="Password"
              placeholder="At least 6 characters"
              autocomplete="new-password"
              :error="passwordError"
              @update:model-value="setPassword"
            />

            <p v-if="error" class="form-error">{{ error }}</p>

            <AppButton type="submit" variant="primary" size="lg" block :disabled="busy">
              {{ busy ? 'Creating account…' : 'Create account' }}
            </AppButton>
          </form>

          <p class="auth-alt">
            Already registered?
            <router-link class="auth-link" to="/login">Sign in</router-link>
          </p>
        </div>
      </div>
    </ion-content>
  </ion-page>
</template>

<style scoped>
.reg-bg {
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