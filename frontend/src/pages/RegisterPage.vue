<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { IonContent, IonIcon, IonPage } from '@ionic/vue'
import { walletOutline } from 'ionicons/icons'
import { useAuthStore } from '@/stores/authStore'
import AppButton from '@/components/ui/AppButton.vue'
import AppInput from '@/components/ui/AppInput.vue'

const router = useRouter()
const auth = useAuthStore()

const name = ref('')
const email = ref('')
const password = ref('')
const busy = ref(false)
const error = ref('')

async function submit() {
  if (!name.value.trim() || !email.value.trim() || !password.value) {
    error.value = 'All three fields are required.'
    return
  }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.value.trim())) {
    error.value = 'Enter a valid email address.'
    return
  }
  if (password.value.length < 6) {
    error.value = 'Password must be at least 6 characters.'
    return
  }
  busy.value = true
  error.value = ''
  try {
    await auth.register(email.value.trim(), name.value.trim(), password.value)
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
          <div class="auth-mark"><ion-icon :icon="walletOutline" /></div>
          <h1 class="auth-title">Create your account</h1>
          <p class="auth-sub">Start tracking banks, deposits and investments</p>

          <form class="form-grid" @submit.prevent="submit">
            <AppInput v-model="name" label="Name" placeholder="Your name" autocomplete="name" />
            <AppInput v-model="email" type="email" label="Email" placeholder="you@example.com" autocomplete="email" />
            <AppInput
              v-model="password"
              type="password"
              label="Password"
              placeholder="At least 6 characters"
              autocomplete="new-password"
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
  font-size: 1.35rem;
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