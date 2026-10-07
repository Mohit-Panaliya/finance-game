<template>
  <ion-page>
    <ion-header :translucent="true">
      <ion-toolbar>
        <ion-buttons slot="start">
          <ion-back-button default-href="/accounts/assets"></ion-back-button>
        </ion-buttons>
        <ion-title>Depreciation</ion-title>
      </ion-toolbar>
    </ion-header>
    <ion-content :fullscreen="true">
      <div class="content-container" v-if="depr">
        <ion-card>
          <ion-card-header>
            <ion-card-title>{{ asset?.name || assetId }}</ion-card-title>
          </ion-card-header>
          <ion-card-content>
            <div class="summary">
              <div><strong>Method:</strong> {{ depr.method }}</div>
              <div><strong>Cost:</strong> {{ depr.cost }}</div>
              <div><strong>Salvage:</strong> {{ depr.salvage_value }}</div>
              <div><strong>Useful life (months):</strong> {{ depr.useful_life_months }}</div>
              <div><strong>Schedule start:</strong> {{ depr.schedule_start }}</div>
              <div><strong>Book value now:</strong> {{ depr.book_value }}</div>
              <div><strong>Accumulated:</strong> {{ depr.accumulated_depreciation }}</div>
              <div><strong>Fully depreciated:</strong> {{ depr.fully_depreciated_on || '-' }}</div>
            </div>
            <app-button @click="rebuild">Rebuild schedule</app-button>
          </ion-card-content>
        </ion-card>

        <ion-card>
          <ion-card-header><ion-card-title>Schedule</ion-card-title></ion-card-header>
          <ion-card-content>
            <div class="table-wrapper">
              <table class="depr-table">
                <thead>
                  <tr>
                    <th>Period</th>
                    <th>Start → End</th>
                    <th>Opening</th>
                    <th>Charge</th>
                    <th>Closing</th>
                    <th>Accumulated</th>
                  </tr>
                </thead>
                <tbody>
                  <tr v-for="p in depr.periods || []" :key="p.index">
                    <td>{{ p.index }}</td>
                    <td>{{ p.start }} → {{ p.end }}</td>
                    <td class="num">{{ p.opening }}</td>
                    <td class="num">{{ p.charge }}</td>
                    <td class="num">{{ p.closing }}</td>
                    <td class="num">{{ p.accumulated }}</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </ion-card-content>
        </ion-card>
      </div>
    </ion-content>
  </ion-page>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute } from 'vue-router'
import {
  IonPage, IonHeader, IonToolbar, IonTitle, IonContent, IonCard, IonCardHeader, IonCardTitle,
  IonCardContent, IonBackButton, IonButtons
} from '@ionic/vue'
import AppButton from '@/components/ui/AppButton.vue'
import { request } from '@/api/client'

const route = useRoute()
const assetId = computed(() => route.params.id as string)
const depr = ref<any>(null)
const asset = ref<any>(null)

async function load() {
  try {
    depr.value = await request(`/api/assets/${assetId.value}/depreciation`, { method: 'GET' })
  } catch (e) { /* ignore */ }
  try {
    asset.value = await request(`/api/assets/${assetId.value}`, { method: 'GET' })
  } catch (e) { /* ignore */ }
}

async function rebuild() {
  await request(`/api/assets/${assetId.value}/depreciation`, { method: 'POST' })
  await load()
}

onMounted(load)
</script>

<style scoped>
.content-container { padding: var(--density-page-pad); }
.summary { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: var(--density-gap); margin-bottom: var(--density-gap); }
.table-wrapper { overflow-x: auto; }
.depr-table { width: 100%; border-collapse: collapse; }
.depr-table th, .depr-table td { border: 1px solid var(--border); padding: var(--density-row-pad-y); text-align: left; }
.depr-table .num { font-variant-numeric: tabular-nums; text-align: right; }
</style>
