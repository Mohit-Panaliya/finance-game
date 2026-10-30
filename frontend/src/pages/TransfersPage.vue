<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { IonContent, IonPage, IonHeader, IonToolbar, IonTitle, IonButtons, IonButton, IonIcon, IonList, IonItem, IonLabel, IonSelect, IonSelectOption, IonInput, IonModal, IonCard, IonCardContent } from '@ionic/vue'
import { addOutline, swapHorizontalOutline, closeOutline } from 'ionicons/icons'
import { useFinanceStore } from '@/stores/financeStore'
import { request } from '@/api/client'
import { formatMoney } from '@/utils/money'
import AppButton from '@/components/ui/AppButton.vue'

const finance = useFinanceStore()

interface Transfer {
  id: string
  from_bank_id: string
  to_bank_id: string
  amount: number
  currency: string
  transfer_date: string
  description: string | null
  created_at: string
}

const transfers = ref<Transfer[]>([])
const loading = ref(false)
const showModal = ref(false)
const fromBank = ref('')
const toBank = ref('')
const amount = ref('')
const description = ref('')

const banks = computed(() => finance.rows('banks'))

async function fetchTransfers() {
  loading.value = true
  try {
    transfers.value = await request<Transfer[]>('/transfers')
  } catch (e) {
    console.error('Failed to fetch transfers', e)
  } finally {
    loading.value = false
  }
}

async function createTransfer() {
  if (!fromBank.value || !toBank.value || !amount.value) return
  if (fromBank.value === toBank.value) {
    alert('Source and destination must be different')
    return
  }
  try {
    await request('/transfers', {
      method: 'POST',
      body: {
        from_bank_id: fromBank.value,
        to_bank_id: toBank.value,
        amount: parseFloat(amount.value),
        description: description.value || null
      }
    })
    showModal.value = false
    fromBank.value = ''
    toBank.value = ''
    amount.value = ''
    description.value = ''
    await fetchTransfers()
    await finance.fetchList('banks')
  } catch (e) {
    console.error('Failed to create transfer', e)
    alert('Transfer failed')
  }
}

function bankName(id: string): string {
  const bank = banks.value.find((b: any) => b.id === id)
  return bank ? bank.name : id
}

onMounted(() => {
  finance.fetchList('banks')
  fetchTransfers()
})
</script>

<template>
  <IonPage>
    <IonHeader>
      <IonToolbar>
        <IonTitle>Transfers</IonTitle>
        <IonButtons slot="end">
          <IonButton @click="showModal = true">
            <IonIcon :icon="addOutline" />
          </IonButton>
        </IonButtons>
      </IonToolbar>
    </IonHeader>
    <IonContent>
      <IonList>
        <IonItem v-for="t in transfers" :key="t.id">
          <IonIcon :icon="swapHorizontalOutline" slot="start" />
          <IonLabel>
            <h2>{{ bankName(t.from_bank_id) }} → {{ bankName(t.to_bank_id) }}</h2>
            <p>{{ t.transfer_date }}{{ t.description ? ' · ' + t.description : '' }}</p>
          </IonLabel>
          <span slot="end" class="amount">{{ formatMoney(t.amount, t.currency) }}</span>
        </IonItem>
      </IonList>
      <div v-if="!transfers.length && !loading" class="empty">
        No transfers yet. Tap + to create one.
      </div>
    </IonContent>

    <IonModal :is-open="showModal" @didDismiss="showModal = false">
      <IonHeader>
        <IonToolbar>
          <IonTitle>New Transfer</IonTitle>
          <IonButtons slot="end">
            <IonButton @click="showModal = false">
              <IonIcon :icon="closeOutline" />
            </IonButton>
          </IonButtons>
        </IonToolbar>
      </IonHeader>
      <IonContent class="ion-padding">
        <IonCard>
          <IonCardContent>
            <IonSelect v-model="fromBank" label="From account" label-placement="stacked" interface="popover">
              <IonSelectOption v-for="b in banks" :key="b.id" :value="b.id">{{ b.name }} ({{ formatMoney(b.current_balance, b.currency) }})</IonSelectOption>
            </IonSelect>
            <IonSelect v-model="toBank" label="To account" label-placement="stacked" interface="popover">
              <IonSelectOption v-for="b in banks" :key="b.id" :value="b.id">{{ b.name }} ({{ formatMoney(b.current_balance, b.currency) }})</IonSelectOption>
            </IonSelect>
            <IonInput v-model="amount" type="number" label="Amount" label-placement="stacked" />
            <IonInput v-model="description" label="Description (optional)" label-placement="stacked" />
            <AppButton @click="createTransfer" style="margin-top: 16px;">Transfer</AppButton>
          </IonCardContent>
        </IonCard>
      </IonContent>
    </IonModal>
  </IonPage>
</template>

<style scoped>
.amount {
  font-weight: 600;
}
.empty {
  text-align: center;
  padding: 40px 20px;
  color: var(--ion-color-medium);
}
</style>
