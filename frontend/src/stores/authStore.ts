import { defineStore } from 'pinia'
import { api, ApiError, getToken, setToken } from '@/api/client'
import type { User } from '@/types'

interface AuthState {
  user: User | null
  loading: boolean
  error: string | null
  ready: boolean
}

export const useAuthStore = defineStore('auth', {
  state: (): AuthState => ({
    user: null,
    loading: false,
    error: null,
    ready: false
  }),

  getters: {
    isAuthenticated: (s): boolean => !!s.user && !!getToken()
  },

  actions: {
    async init() {
      if (this.ready) return
      this.ready = true
      if (getToken()) {
        try {
          const me = await api.get<User | { user?: User }>('/auth/me')
          this.user = (me && 'user' in me ? (me.user ?? null) : (me as User | null)) ?? null
        } catch {
          this.user = null
          setToken(null)
        }
      }
    },

    async login(username: string, password: string) {
      this.loading = true
      this.error = null
      try {
        const res = await api.post<{ token?: string; jwt?: string; user?: User }>('/auth/login', {
          username,
          password
        })
        const token = res?.token ?? res?.jwt ?? null
        if (token) setToken(token)
        this.user = res?.user ?? { username }
        if (!token) await this.fetchMe()
        return true
      } catch (e) {
        this.error = e instanceof ApiError ? e.message : 'Login failed'
        throw e
      } finally {
        this.loading = false
      }
    },

    async register(email: string, name: string, password: string) {
      this.loading = true
      this.error = null
      try {
        const res = await api.post<{ token?: string; jwt?: string; user?: User }>('/auth/register', {
          email,
          name,
          password
        })
        const token = res?.token ?? res?.jwt ?? null
        if (token) setToken(token)
        this.user = res?.user ?? { email, name }
        if (!token) await this.fetchMe()
        return true
      } catch (e) {
        this.error = e instanceof ApiError ? e.message : 'Registration failed'
        throw e
      } finally {
        this.loading = false
      }
    },

    async fetchMe() {
      try {
        const me = await api.get<User | { user?: User }>('/auth/me')
        this.user = (me && 'user' in me ? (me.user ?? null) : (me as User | null)) ?? null
      } catch (e) {
        if (e instanceof ApiError && e.status === 401) {
          this.user = null
          setToken(null)
        }
      }
    },

    logout() {
      this.user = null
      setToken(null)
    }
  }
})
