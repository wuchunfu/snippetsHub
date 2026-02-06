<!--
  应用更新检查组件
  自动检查并提示用户下载新版本
  作者：开发团队
  更新：2026-02
-->
<template>
  <Teleport to="body">
    <!-- 更新提示对话框 -->
    <Transition name="fade">
      <div v-if="showUpdateDialog" class="update-overlay" @click.self="closeDialog">
        <div class="update-dialog">
          <div class="dialog-header">
            <h3>🎉 发现新版本</h3>
            <button @click="closeDialog" class="close-btn">
              <X :size="20" />
            </button>
          </div>

          <div class="dialog-body">
            <div class="version-info">
              <div class="version-row">
                <span class="label">当前版本：</span>
                <span class="value">v{{ currentVersion }}</span>
              </div>
              <div class="version-row">
                <span class="label">最新版本：</span>
                <span class="value highlight">v{{ latestVersion }}</span>
              </div>
            </div>

            <div v-if="updateNotes" class="update-notes">
              <h4>更新内容</h4>
              <div class="notes-content" v-html="updateNotes"></div>
            </div>

            <div class="update-tips">
              <p>✨ 更新后，您的所有数据都会自动保留</p>
              <p>📝 包括代码片段、TODO 列表、Markdown 文档等</p>
            </div>

            <div v-if="downloading" class="download-progress">
              <div class="progress-bar">
                <div class="progress-fill" :style="{ width: downloadProgress + '%' }"></div>
              </div>
              <p class="progress-text">正在下载更新... {{ downloadProgress }}%</p>
            </div>
          </div>

          <div class="dialog-footer">
            <button @click="closeDialog" class="btn-secondary">
              稍后提醒
            </button>
            <button @click="installUpdate" class="btn-primary" :disabled="downloading">
              {{ downloading ? '下载中...' : '立即更新' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { X } from 'lucide-vue-next'

// 响应式状态
const showUpdateDialog = ref(false) // 是否显示更新对话框
const currentVersion = ref('1.0.0') // 当前版本号
const latestVersion = ref('') // 最新版本号
const updateNotes = ref('') // 更新说明
const downloading = ref(false) // 是否正在下载
const downloadProgress = ref(0) // 下载进度

/**
 * 检查应用更新
 * 会在应用启动时自动调用，也可以手动调用
 */
const checkForUpdates = async () => {
  try {
    console.log('开始检查更新...')
    
    // 使用 Tauri 更新器插件检查更新
    const update = await check()
    
    if (update) {
      console.log('发现新版本:', update.version)
      latestVersion.value = update.version
      updateNotes.value = update.body || '暂无更新说明'
      showUpdateDialog.value = true
      
      // 保存更新对象供后续使用
      window.__pendingUpdate = update
    } else {
      console.log('已是最新版本')
    }
  } catch (error) {
    console.error('检查更新失败:', error)
    
    // 如果是配置错误，静默失败（开发环境常见）
    // 如果是网络错误，也静默失败
    // 只在控制台输出日志供开发者调试
    if (error.message && error.message.includes('GITHUB_REPOSITORY')) {
      console.info('提示：更新功能需要配置 GitHub 仓库地址')
      console.info('请在 tauri.conf.json 中配置正确的 endpoints')
    }
  }
}

/**
 * 安装更新
 * 下载并安装新版本，完成后重启应用
 */
const installUpdate = async () => {
  const update = window.__pendingUpdate
  if (!update) {
    console.error('没有待安装的更新')
    return
  }

  try {
    downloading.value = true
    downloadProgress.value = 0

    console.log('开始下载更新...')
    
    // 下载并安装更新
    // Tauri 会自动处理下载和安装过程
    await update.downloadAndInstall((event) => {
      switch (event.event) {
        case 'Started':
          console.log('下载开始')
          downloadProgress.value = 0
          break
        case 'Progress':
          // 更新下载进度
          if (event.data.contentLength) {
            const progress = Math.round((event.data.downloaded / event.data.contentLength) * 100)
            downloadProgress.value = progress
            console.log(`下载进度: ${progress}%`)
          }
          break
        case 'Finished':
          console.log('下载完成')
          downloadProgress.value = 100
          break
      }
    })

    console.log('更新安装完成，准备重启应用...')
    
    // 给用户一点时间看到完成提示
    setTimeout(async () => {
      // 重启应用以应用更新
      await relaunch()
    }, 1000)
    
  } catch (error) {
    console.error('更新安装失败:', error)
    alert('更新安装失败，请稍后重试')
    downloading.value = false
  }
}

/**
 * 关闭更新对话框
 */
const closeDialog = () => {
  if (!downloading.value) {
    showUpdateDialog.value = false
  }
}

/**
 * 手动触发更新检查
 * 可以从设置页面调用此方法
 */
const manualCheck = async () => {
  await checkForUpdates()
  if (!showUpdateDialog.value) {
    // 如果没有更新，显示提示
    alert('当前已是最新版本')
  }
}

// 应用启动时自动检查更新
onMounted(() => {
  // 延迟 3 秒后检查，避免影响应用启动速度
  setTimeout(() => {
    checkForUpdates()
  }, 3000)
})

// 导出方法供外部调用
defineExpose({
  checkForUpdates,
  manualCheck
})
</script>

<style scoped>
.update-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
  backdrop-filter: blur(4px);
}

.update-dialog {
  background: var(--color-background);
  border-radius: 12px;
  width: 90%;
  max-width: 500px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.close-btn {
  padding: 4px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.dialog-body {
  padding: 24px;
  max-height: 400px;
  overflow-y: auto;
}

.version-info {
  margin-bottom: 20px;
}

.version-row {
  display: flex;
  align-items: center;
  padding: 8px 0;
  font-size: 14px;
}

.version-row .label {
  color: var(--color-text-secondary);
  min-width: 80px;
}

.version-row .value {
  color: var(--color-text-primary);
  font-weight: 500;
}

.version-row .value.highlight {
  color: var(--color-primary);
  font-weight: 600;
}

.update-notes {
  margin-bottom: 20px;
  padding: 16px;
  background: var(--color-background-secondary);
  border-radius: 8px;
}

.update-notes h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--color-text-primary);
}

.notes-content {
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-secondary);
}

.update-tips {
  padding: 12px 16px;
  background: var(--color-success-bg, #e8f5e9);
  border-radius: 8px;
  margin-bottom: 20px;
}

.update-tips p {
  margin: 4px 0;
  font-size: 13px;
  color: var(--color-success, #2e7d32);
}

.download-progress {
  margin-top: 16px;
}

.progress-bar {
  height: 8px;
  background: var(--color-background-secondary);
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 8px;
}

.progress-fill {
  height: 100%;
  background: var(--color-primary);
  transition: width 0.3s ease;
  border-radius: 4px;
}

.progress-text {
  text-align: center;
  font-size: 13px;
  color: var(--color-text-secondary);
  margin: 0;
}

.dialog-footer {
  display: flex;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--color-border);
  background: var(--color-background-secondary);
}

.btn-secondary,
.btn-primary {
  flex: 1;
  padding: 10px 20px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.btn-secondary {
  background: transparent;
  color: var(--color-text-secondary);
  border: 1px solid var(--color-border);
}

.btn-secondary:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

.btn-primary {
  background: var(--color-primary);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

/* 淡入淡出动画 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
