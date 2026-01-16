<script setup lang="ts">
import { ref, watch, computed } from 'vue'
import {
  NModal,
  NForm,
  NFormItem,
  NInput,
  NButton,
  NSpace,
  useMessage
} from 'naive-ui'
import { api } from '@/api/adapter'
import type { K8sFavorite, CreateK8sFavoriteRequest, UpdateK8sFavoriteRequest } from '@/api/types'

const props = defineProps<{
  show: boolean
  clusterId: number | null
  clusterName: string
  namespace: string | null
  editingFavorite?: K8sFavorite | null
}>()

const emit = defineEmits<{
  (e: 'update:show', value: boolean): void
  (e: 'saved'): void
}>()

const message = useMessage()
const saving = ref(false)

const form = ref({
  name: '',
  description: '',
  category: ''
})

const isEditing = computed(() => !!props.editingFavorite)

const modalTitle = computed(() =>
  isEditing.value ? 'Edit Favorite' : 'Add to Favorites'
)

// Initialize form when editing or when modal opens
watch(() => props.show, (show) => {
  if (show) {
    if (props.editingFavorite) {
      form.value = {
        name: props.editingFavorite.name,
        description: props.editingFavorite.description || '',
        category: props.editingFavorite.category || ''
      }
    } else {
      // Default name combining cluster and namespace
      form.value = {
        name: `${props.clusterName}-${props.namespace}`,
        description: '',
        category: ''
      }
    }
  }
})

async function handleSave() {
  if (!form.value.name.trim()) {
    message.warning('Please enter a name')
    return
  }

  if (!props.clusterId || !props.namespace) {
    message.error('Please select a cluster and namespace first')
    return
  }

  saving.value = true
  try {
    if (isEditing.value && props.editingFavorite?.id) {
      // Update existing favorite
      const request: UpdateK8sFavoriteRequest = {
        name: form.value.name.trim(),
        description: form.value.description.trim() || undefined,
        category: form.value.category.trim() || undefined
      }
      await api.k8sFavorite.update(props.editingFavorite.id, request)
      message.success('Favorite updated')
    } else {
      // Create new favorite
      const request: CreateK8sFavoriteRequest = {
        name: form.value.name.trim(),
        cluster_id: props.clusterId,
        namespace: props.namespace,
        description: form.value.description.trim() || undefined,
        category: form.value.category.trim() || undefined
      }
      await api.k8sFavorite.create(request)
      message.success('Added to favorites')
    }
    emit('update:show', false)
    emit('saved')
  } catch (error) {
    message.error('Failed to save: ' + (error as Error).message)
  } finally {
    saving.value = false
  }
}

function handleClose() {
  emit('update:show', false)
}
</script>

<template>
  <NModal
    :show="show"
    preset="card"
    :title="modalTitle"
    style="width: 420px"
    :mask-closable="false"
    @update:show="emit('update:show', $event)"
  >
    <NForm :model="form" label-placement="top">
      <NFormItem label="Name" required>
        <NInput
          v-model:value="form.name"
          placeholder="e.g. Game UAT"
        />
      </NFormItem>

      <NFormItem label="Description">
        <NInput
          v-model:value="form.description"
          type="textarea"
          placeholder="Optional description"
          :rows="2"
        />
      </NFormItem>

      <NFormItem label="Category">
        <NInput
          v-model:value="form.category"
          placeholder="Optional category for grouping"
        />
      </NFormItem>

      <div v-if="!isEditing" class="info-text">
        <div><strong>Cluster:</strong> {{ clusterName }}</div>
        <div><strong>Namespace:</strong> {{ namespace }}</div>
      </div>
    </NForm>

    <template #footer>
      <NSpace justify="end">
        <NButton @click="handleClose">Cancel</NButton>
        <NButton type="primary" :loading="saving" @click="handleSave">
          {{ isEditing ? 'Update' : 'Save' }}
        </NButton>
      </NSpace>
    </template>
  </NModal>
</template>

<style scoped>
.info-text {
  font-size: 13px;
  color: var(--n-text-color-3);
  background: var(--n-color-embedded);
  padding: 8px 12px;
  border-radius: 6px;
}

.info-text div {
  margin: 4px 0;
}

.info-text strong {
  color: var(--n-text-color-2);
}
</style>
