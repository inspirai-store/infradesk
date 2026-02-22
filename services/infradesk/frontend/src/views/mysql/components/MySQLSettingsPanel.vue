<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  NCard,
  NSpace,
  NButton,
  NIcon,
  NInputNumber,
  NText,
  useMessage,
  NSlider,
} from 'naive-ui'
import {
  CheckmarkOutline,
} from '@vicons/ionicons5'
import { useMySQLStore } from '@/stores/mysql'

interface Props {
  show: boolean
}

interface Emits {
  (e: 'update:show', value: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const store = useMySQLStore()
const message = useMessage()

// 本地状态
const localQueryLimit = ref(store.queryLimit)
const isSaving = ref(false)

// 是否有效值
const isValidLimit = computed(() => {
  return localQueryLimit.value >= 10 && localQueryLimit.value <= 1000
})

// 保存设置
async function saveSettings() {
  if (!isValidLimit.value) {
    message.warning('查询限制必须在 10 到 1000 之间')
    return
  }

  isSaving.value = true
  try {
    const success = await store.saveQueryLimit(localQueryLimit.value)

    if (success) {
      message.success('查询限制已更新')
      emit('update:show', false)
    } else {
      message.error('保存失败')
    }
  } catch (e) {
    message.error('保存失败')
  } finally {
    isSaving.value = false
  }
}

// 取消
function cancel() {
  // 重置为当前值
  localQueryLimit.value = store.queryLimit
  emit('update:show', false)
}

// 当显示时同步当前值
function onShow() {
  if (props.show) {
    localQueryLimit.value = store.queryLimit
  }
}

// 监听 show 变化
onMounted(() => {
  onShow()
})
</script>

<template>
  <div v-if="show" class="settings-overlay" @click.self="cancel">
    <NCard class="settings-card" title="MySQL 查询设置">
      <template #header-extra>
        <NButton text quaternary @click="cancel">
          <template #icon>
            <NIcon>
              <CheckmarkOutline />
            </NIcon>
          </template>
        </NButton>
      </template>

      <NSpace vertical :size="20">
        <!-- 查询限制设置 -->
        <div class="setting-item">
          <NSpace vertical :size="8">
            <NSpace align="center">
              <NText strong>查询行数限制</NText>
              <NText depth="3" style="font-size: 11px">(防止意外查询大量数据)</NText>
            </NSpace>

            <NSpace vertical :size="4">
              <NSlider
                v-model:value="localQueryLimit"
                :min="10"
                :max="1000"
                :step="10"
                :marks="{ 10: '10', 100: '100', 500: '500', 1000: '1000' }"
              />

              <NSpace align="center" justify="center">
                <NInputNumber
                  v-model:value="localQueryLimit"
                  :min="10"
                  :max="1000"
                  :update-value-on-input="true"
                  style="width: 120px"
                />
                <NText>行</NText>
              </NSpace>
            </NSpace>

            <NText v-if="!isValidLimit" type="warning" style="font-size: 11px">
              限制必须在 10 到 1000 之间
            </NText>

            <NText depth="3" style="font-size: 11px">
              执行 SELECT 查询时，如果没有手动添加 LIMIT 子句，系统将自动添加 LIMIT {{ localQueryLimit }}。
            </NText>
          </NSpace>
        </div>

        <!-- 操作按钮 -->
        <NSpace justify="end" :size="8">
          <NButton @click="cancel">取消</NButton>
          <NButton
            type="primary"
            :loading="isSaving"
            :disabled="!isValidLimit"
            @click="saveSettings"
          >
            保存设置
          </NButton>
        </NSpace>
      </NSpace>
    </NCard>
  </div>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.settings-card {
  width: 400px;
  max-width: 90vw;
}

.setting-item {
  padding: 8px 0;
}
</style>
