<!--
  Markdown 编辑器组件
  基于 Vditor 实现，支持三种编辑模式
  作者：开发团队
  更新：2026-02
-->
<template>
  <div class="markdown-editor-container">
    <!-- 头部工具栏：包含返回按钮、文档标题、保存状态、模式切换等 -->
    <header class="editor-header">
      <!-- 左侧区域：返回按钮 + 文档标题 -->
      <div class="header-left">
        <!-- 返回按钮，点击后返回上一页 -->
        <button @click="$emit('back')" class="icon-btn-ghost" title="返回 (Esc)">
          <ArrowLeft :size="18" />
        </button>

        <div class="divider-vertical"></div>

        <!-- 文档标题输入框，失焦时自动保存 -->
        <input
          v-model="markdownStore.documentTitle"
          class="document-title-input"
          placeholder="无标题文档"
          @blur="markdownStore.saveContent()"
        />
      </div>

      <!-- 右侧区域：保存状态 + 模式切换 + 功能按钮 -->
      <div class="header-right">
        <!-- 保存状态提示 -->
        <span v-if="markdownStore.hasUnsavedChanges" class="save-status">
          <div class="unsaved-dot"></div>
          未保存
        </span>
        <span v-else-if="markdownStore.lastSaved" class="save-status saved">
          已保存 {{ markdownStore.lastSaved }}
        </span>

        <!-- 编辑模式切换按钮组 -->
        <div class="mode-switcher">
          <button 
            @click="switchMode('wysiwyg')" 
            class="mode-btn" 
            :class="{ active: editorMode === 'wysiwyg' }"
            title="分屏预览模式"
          >
            分屏
          </button>
          <button 
            @click="switchMode('ir')" 
            class="mode-btn" 
            :class="{ active: editorMode === 'ir' }"
            title="即时渲染模式"
          >
            即时
          </button>
          <button 
            @click="switchMode('sv')" 
            class="mode-btn" 
            :class="{ active: editorMode === 'sv' }"
            title="源码模式"
          >
            源码
          </button>
        </div>

        <div class="divider-vertical"></div>

        <button @click="toggleOutline" class="icon-btn-ghost" :class="{ active: showOutline }" title="目录大纲">
          <List :size="18" />
        </button>

        <button @click="exportDocument" class="icon-btn-ghost" title="导出">
          <Download :size="18" />
        </button>

        <button @click="showHistory = true" class="icon-btn-ghost" title="历史记录">
          <Clock :size="18" />
        </button>
      </div>
    </header>

    <!-- 编辑器主体 -->
    <div class="editor-body">
      <!-- Vditor 编辑器容器 -->
      <div ref="vditorRef" class="vditor-container"></div>

      <!-- 目录大纲 -->
      <MarkdownOutline
        v-if="showOutline"
        :content="markdownStore.content"
        @close="showOutline = false"
        @scroll-to="scrollToHeading"
      />
    </div>

    <!-- 历史记录侧边栏 -->
    <Teleport to="body">
      <Transition name="slide-right">
        <div v-if="showHistory" class="history-sidebar">
          <div class="history-header">
            <h3>历史记录</h3>
            <button @click="showHistory = false" class="btn-close">
              <X :size="16" />
            </button>
          </div>
          
          <div class="history-list">
            <div
              v-for="snapshot in markdownStore.snapshots"
              :key="snapshot.id"
              class="history-item"
              @click="restoreSnapshot(snapshot)"
            >
              <div class="history-title">{{ snapshot.title }}</div>
              <div class="history-time">{{ formatTime(snapshot.timestamp) }}</div>
              <div class="history-summary">{{ snapshot.summary }}</div>
              <button
                @click.stop="markdownStore.deleteSnapshot(snapshot.id)"
                class="btn-delete"
                title="删除"
              >
                <Trash2 :size="14" />
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- 导出对话框 -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showExportDialog" class="dialog-overlay" @click="showExportDialog = false">
          <div class="dialog-content" @click.stop>
            <h3>导出文档</h3>
            <div class="export-options">
              <button @click="doExport('markdown')" class="export-btn">
                <FileText :size="18" />
                导出为 Markdown
              </button>
              <button @click="doExport('html')" class="export-btn">
                <Code :size="18" />
                导出为 HTML
              </button>
              <button @click="doExport('pdf')" class="export-btn">
                <FileDown :size="18" />
                导出为 PDF
              </button>
            </div>
            <button @click="showExportDialog = false" class="btn-cancel">取消</button>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useMarkdownStore } from '../stores/markdownStore'
import {
  ArrowLeft, Download, Clock, List, X, FileText, Code, FileDown, Trash2
} from 'lucide-vue-next'
import MarkdownOutline from './MarkdownOutline.vue'
import Vditor from 'vditor'
import 'vditor/dist/index.css'

// 定义组件的 emit 事件
const emit = defineEmits(['back'])

// 使用 Pinia store 管理 Markdown 状态
const markdownStore = useMarkdownStore()

// 编辑器相关的响应式引用
const vditorRef = ref(null) // Vditor 容器 DOM 引用
const vditor = ref(null) // Vditor 实例引用
const showOutline = ref(false) // 是否显示目录大纲
const showHistory = ref(false) // 是否显示历史记录侧边栏
const showExportDialog = ref(false) // 是否显示导出对话框
const editorMode = ref('wysiwyg') // 当前编辑模式：wysiwyg(分屏) / ir(即时) / sv(源码)

// 组件挂载后初始化编辑器
onMounted(async () => {
  // 等待 DOM 更新完成
  await nextTick()
  
  // 从 localStorage 加载之前保存的文档数据
  markdownStore.initialize()
  markdownStore.loadSnapshots() // 加载历史快照

  // 初始化 Vditor 编辑器（需要确保 DOM 已渲染）
  if (vditorRef.value) {
    // 创建 Vditor 实例，配置编辑器参数
    vditor.value = new Vditor(vditorRef.value, {
      height: 'calc(100vh - 60px)', // 高度为视口高度减去顶部工具栏
      mode: 'wysiwyg', // 默认使用所见即所得模式（分屏预览）
      placeholder: '开始写作...',
      theme: 'classic', // 使用经典主题
      icon: 'material', // Material Design 风格图标
      width: '100%',
      
      // 工具栏配置
      toolbar: [
        'emoji',
        'headings',
        'bold',
        'italic',
        'strike',
        '|',
        'line',
        'quote',
        'list',
        'ordered-list',
        'check',
        '|',
        'code',
        'inline-code',
        'link',
        'table',
        '|',
        'undo',
        'redo',
        '|',
        'upload',
        'record',
        '|',
        'edit-mode',
        'outline',
        'preview',
        'fullscreen',
        '|',
        'help'
      ],

      // 缓存配置
      cache: {
        enable: false // 使用我们自己的 store 管理
      },

      // 上传配置
      upload: {
        accept: 'image/*,.mp3, .wav, .ogg',
        multiple: false,
        filename(name) {
          return name.replace(/[^(a-zA-Z0-9\u4e00-\u9fa5\.)]/g, '')
            .replace(/[\?\\/:|<>\*\[\]\(\)\$%\{\}@~]/g, '')
            .replace('/\\s/g', '')
        },
        handler(files) {
          // 这里可以集成文件上传功能
          console.log('Files to upload:', files)
          return null
        }
      },

      // 计数器
      counter: {
        enable: true,
        type: 'markdown'
      },

      // 大纲配置
      outline: {
        enable: false // 使用我们自己的大纲组件
      },

      // 预览配置
      preview: {
        delay: 300,
        mode: 'both', // 分屏显示：编辑器和预览并排
        hljs: {
          enable: true,
          lineNumber: true,
          style: 'github'
        },
        markdown: {
          toc: true,
          mark: true,
          footnotes: true,
          autoSpace: true,
          linkBase: '',
          linkPrefix: ''
        },
        math: {
          inlineDigit: false,
          engine: 'KaTeX'
        },
        theme: {
          current: 'light',
          path: 'https://cdn.jsdelivr.net/npm/vditor/dist/css/content-theme'
        }
      },
      
      // 编辑器配置
      resize: {
        enable: false
      },
      
      // 所见即所得模式配置
      typewriterMode: false,
      
      // 代码块配置
      tab: '  ',

      // 提示配置
      hint: {
        emoji: {
          '+1': '👍',
          '-1': '👎',
          'confused': '😕',
          'eyes': '👀',
          'heart': '❤️',
          'rocket': '🚀',
          'smile': '😄',
          'tada': '🎉'
        }
      },

      // 输入后回调
      input: (value) => {
        markdownStore.updateContent(value)
      },

      // 聚焦后回调
      focus: (value) => {
        console.log('Editor focused')
      },

      // 失焦后回调
      blur: (value) => {
        markdownStore.saveContent()
      },

      // 编辑器初始化完成后的回调函数
      after: () => {
        // 如果 store 中有保存的内容，恢复到编辑器中
        if (vditor.value && markdownStore.content) {
          vditor.value.setValue(markdownStore.content)
        }
        
        // 将工具栏按钮的 aria-label 复制到 title 属性
        // 这样鼠标悬停时可以显示浏览器原生提示
        nextTick(() => {
          const toolbarItems = vditorRef.value?.querySelectorAll('[aria-label]')
          toolbarItems?.forEach(item => {
            const label = item.getAttribute('aria-label')
            if (label && !item.getAttribute('title')) {
              item.setAttribute('title', label)
            }
          })
        })
      }
    })
  }
})

// 监听 store 中的内容变化
// 如果是通过其他方式（如历史记录恢复）改变了内容，同步到编辑器
watch(() => markdownStore.content, (newContent) => {
  if (vditor.value && vditor.value.getValue() !== newContent) {
    vditor.value.setValue(newContent)
  }
})

/**
 * 切换编辑模式
 * @param {string} mode - 目标模式：'wysiwyg'(分屏) / 'ir'(即时) / 'sv'(源码)
 */
const switchMode = (mode) => {
  // 只有在切换到不同模式时才执行
  if (vditor.value && editorMode.value !== mode) {
    editorMode.value = mode
    // 保存当前编辑器内容，切换模式时不会丢失
    const content = vditor.value.getValue()
    
    // 先销毁旧的编辑器实例（Vditor 模式切换需要重新初始化）
    vditor.value.destroy()
    
    vditor.value = new Vditor(vditorRef.value, {
      height: 'calc(100vh - 60px)',
      mode: mode,
      placeholder: '开始写作...',
      theme: 'classic',
      icon: 'material',
      width: '100%',
      value: content,
      
      toolbar: [
        'emoji', 'headings', 'bold', 'italic', 'strike', '|',
        'line', 'quote', 'list', 'ordered-list', 'check', '|',
        'code', 'inline-code', 'link', 'table', '|',
        'undo', 'redo', '|',
        'upload', 'record', '|',
        'edit-mode', 'outline', 'preview', 'fullscreen', '|',
        'help'
      ],
      
      cache: { enable: false },
      counter: { enable: true, type: 'markdown' },
      outline: { enable: false },
      
      preview: {
        delay: 300,
        mode: 'both',
        hljs: { enable: true, lineNumber: true, style: 'github' },
        markdown: { toc: true, mark: true, footnotes: true, autoSpace: true },
        math: { inlineDigit: false, engine: 'KaTeX' },
        theme: { current: 'light' }
      },
      
      resize: { enable: false },
      typewriterMode: false,
      tab: '  ',
      
      hint: {
        emoji: { '+1': '👍', '-1': '👎', 'confused': '😕', 'eyes': '👀', 'heart': '❤️', 'rocket': '🚀', 'smile': '😄', 'tada': '🎉' }
      },
      
      input: (value) => { markdownStore.updateContent(value) },
      blur: (value) => { markdownStore.saveContent() },
      after: () => {
        if (vditor.value && content) {
          vditor.value.setValue(content)
        }
        
        // 将 aria-label 复制到 title
        nextTick(() => {
          const toolbarItems = vditorRef.value?.querySelectorAll('[aria-label]')
          toolbarItems?.forEach(item => {
            const label = item.getAttribute('aria-label')
            if (label && !item.getAttribute('title')) {
              item.setAttribute('title', label)
            }
          })
        })
      }
    })
  }
}

// 切换目录大纲显示/隐藏
const toggleOutline = () => {
  showOutline.value = !showOutline.value
}

/**
 * 滚动到指定标题位置
 * @param {Object} heading - 标题对象，包含 id、text、level 等信息
 */
const scrollToHeading = (heading) => {
  if (vditor.value) {
    // Vditor 内置的大纲跳转
    const element = vditorRef.value.querySelector(`[data-id="${heading.id}"]`)
    if (element) {
      element.scrollIntoView({ behavior: 'smooth', block: 'start' })
    }
  }
}

// 显示导出对话框
const exportDocument = () => {
  showExportDialog.value = true
}

/**
 * 执行文档导出
 * @param {string} format - 导出格式：'markdown' / 'html' / 'pdf'
 */
const doExport = async (format) => {
  try {
    let content
    let filename
    let mimeType

    switch (format) {
      case 'markdown':
        content = markdownStore.exportMarkdown()
        filename = `${markdownStore.documentTitle || '未命名文档'}.md`
        mimeType = 'text/markdown'
        break

      case 'html':
        if (vditor.value) {
          content = vditor.value.getHTML()
        } else {
          content = markdownStore.convertToHtml()
        }
        filename = `${markdownStore.documentTitle || '未命名文档'}.html`
        mimeType = 'text/html'
        
        // 添加完整的 HTML 结构
        content = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${markdownStore.documentTitle || '未命名文档'}</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/vditor/dist/index.css" />
  <style>
    body { max-width: 900px; margin: 40px auto; padding: 0 20px; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }
  <\/style>
</head>
<body class="vditor-reset">
  ${content}
</body>
</html>`
        break

      case 'pdf':
        // PDF 导出需要使用 html2canvas + jsPDF
        // 暂时导出 HTML，用户可以使用浏览器打印为 PDF
        content = vditor.value ? vditor.value.getHTML() : markdownStore.convertToHtml()
        filename = `${markdownStore.documentTitle || '未命名文档'}.html`
        mimeType = 'text/html'
        content = `<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <title>${markdownStore.documentTitle || '未命名文档'}</title>
  <link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/vditor/dist/index.css" />
  <style>
    @media print { body { max-width: none; } }
    body { max-width: 900px; margin: 40px auto; padding: 0 20px; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif; }
  </style>
</head>
<body class="vditor-reset">
  ${content}
  <script>window.print();<\/script>
</body>
</html>`
        alert('将打开打印对话框，请选择"另存为 PDF"')
        break
    }

    // 创建下载链接
    const blob = new Blob([content], { type: mimeType })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    URL.revokeObjectURL(url)

    showExportDialog.value = false
  } catch (error) {
    console.error('Export failed:', error)
    alert('导出失败：' + error.message)
  }
}

/**
 * 从历史快照恢复文档
 * @param {Object} snapshot - 快照对象，包含内容、标题、时间戳等
 */
const restoreSnapshot = (snapshot) => {
  if (confirm(`确定要恢复到 "${snapshot.title}" 吗？当前未保存的更改将丢失。`)) {
    markdownStore.restoreSnapshot(snapshot)
    if (vditor.value) {
      vditor.value.setValue(snapshot.content)
    }
    showHistory.value = false
  }
}

/**
 * 格式化时间显示
 * @param {string} timestamp - ISO 格式的时间戳
 * @returns {string} 格式化后的时间字符串（如"刚刚"、"5分钟前"等）
 */
const formatTime = (timestamp) => {
  const date = new Date(timestamp)
  const now = new Date()
  const diff = now - date

  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)} 分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)} 小时前`
  if (diff < 604800000) return `${Math.floor(diff / 86400000)} 天前`

  return date.toLocaleDateString('zh-CN')
}

// 组件卸载时清理资源
onUnmounted(() => {
  // 销毁 Vditor 实例，释放内存
  if (vditor.value) {
    vditor.value.destroy()
    vditor.value = null
  }
  // 清理 store 中的定时器等资源
  markdownStore.cleanup()
})
</script>

<style scoped>
.markdown-editor-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--color-background);
}

.editor-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 60px;
  padding: 0 24px;
  background: var(--color-background);
  border-bottom: 1px solid var(--color-border);
}

.header-left,
.header-right {
  display: flex;
  align-items: center;
  gap: 12px;
}

.icon-btn-ghost {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  padding: 0;
  background: transparent;
  border: none;
  border-radius: 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.icon-btn-ghost:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.icon-btn-ghost.active {
  background: var(--color-primary);
  color: white;
}

.divider-vertical {
  width: 1px;
  height: 24px;
  background: var(--color-border);
  margin: 0 8px;
}

.mode-switcher {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 4px;
  background: var(--color-background-secondary);
  border-radius: 8px;
}

.mode-btn {
  padding: 6px 12px;
  font-size: 13px;
  font-weight: 500;
  color: var(--color-text-secondary);
  background: transparent;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.mode-btn:hover {
  background: var(--color-background);
  color: var(--color-text-primary);
}

.mode-btn.active {
  background: var(--color-primary);
  color: white;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.document-title-input {
  padding: 8px 12px;
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  background: transparent;
  border: 1px solid transparent;
  border-radius: 6px;
  outline: none;
  transition: all 0.2s;
  min-width: 200px;
}

.document-title-input:hover {
  background: var(--color-background-secondary);
  border-color: var(--color-border);
}

.document-title-input:focus {
  background: var(--color-background);
  border-color: var(--color-primary);
}

.save-status {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--color-text-secondary);
}

.unsaved-dot {
  width: 6px;
  height: 6px;
  background: var(--color-warning);
  border-radius: 50%;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

.save-status.saved {
  color: var(--color-success);
}

.editor-body {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.vditor-container {
  flex: 1;
  overflow: hidden;
}

/* Vditor 主题定制 */
:deep(.vditor) {
  border: none;
}

:deep(.vditor-toolbar) {
  background: var(--color-background);
  border-bottom: 1px solid var(--color-border);
  padding: 8px 16px;
}

:deep(.vditor-toolbar__item) {
  color: var(--color-text-secondary);
}

:deep(.vditor-toolbar__item:hover) {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

:deep(.vditor-ir) {
  background: var(--color-background);
  color: var(--color-text-primary);
}

:deep(.vditor-ir pre.vditor-reset) {
  background: var(--color-background-secondary);
}

:deep(.vditor-counter) {
  color: var(--color-text-tertiary);
}

/* Vditor 工具栏按钮 - 使用浏览器原生 title 提示 */
:deep(.vditor-toolbar__item) {
  cursor: pointer;
}

/* 确保 Vditor 的 tooltipped 元素不显示自定义提示 */
:deep(.vditor-tooltipped::after),
:deep(.vditor-tooltipped::before) {
  display: none !important;
}

/* 固定白色背景主题 - 所有模式统一 */
:deep(.vditor-wysiwyg) {
  background: #ffffff !important;
}

:deep(.vditor-sv) {
  background: #ffffff !important;
}

:deep(.vditor-preview) {
  background: #ffffff !important;
}

:deep(.vditor-ir) {
  background: #ffffff !important;
}

/* 即时渲染模式编辑区域 - 完全覆盖 */
:deep(.vditor-ir .vditor-reset) {
  background: #ffffff !important;
}

:deep(.vditor-ir pre.vditor-reset) {
  background: #ffffff !important;
  color: #24292e !important;
}

:deep(.vditor-ir .vditor-ir__node) {
  background: #ffffff !important;
}

:deep(.vditor-ir .vditor-ir__marker) {
  background: #ffffff !important;
}

:deep(.vditor-ir__preview) {
  background: #ffffff !important;
}

/* 即时渲染输入区域 */
:deep(.vditor-ir pre[contenteditable="true"]) {
  background: #ffffff !important;
  color: #24292e !important;
}

/* 基础文字颜色 */
:deep(.vditor-reset) {
  color: #24292e !important;
}

/* 标题样式 */
:deep(.vditor-reset h1),
:deep(.vditor-reset h2),
:deep(.vditor-reset h3),
:deep(.vditor-reset h4),
:deep(.vditor-reset h5),
:deep(.vditor-reset h6) {
  color: #24292e !important;
  font-weight: 700 !important;
}

/* 段落文本 */
:deep(.vditor-reset p),
:deep(.vditor-reset li) {
  color: #24292e !important;
}

/* 行内代码 - 保持语法高亮 */
:deep(.vditor-reset code:not(pre code)) {
  background: #f6f8fa !important;
  color: #e36209 !important;
  padding: 2px 6px;
  border-radius: 3px;
}

/* 代码块 - 不覆盖语法高亮颜色 */
:deep(.vditor-reset pre) {
  background: #f6f8fa !important;
  border: 1px solid #e1e4e8 !important;
}

:deep(.vditor-reset pre code) {
  background: transparent !important;
  /* 不设置 color，让 highlight.js 的语法高亮生效 */
}

/* 链接样式 */
:deep(.vditor-reset a) {
  color: #0969da !important;
}

/* 引用块样式 */
:deep(.vditor-reset blockquote) {
  color: #57606a !important;
  border-left-color: #d0d7de !important;
}

/* 表格样式 */
:deep(.vditor-reset table) {
  color: #24292e !important;
}

:deep(.vditor-reset th) {
  background: #f6f8fa !important;
  border-color: #d0d7de !important;
  color: #24292e !important;
}

:deep(.vditor-reset td) {
  border-color: #d0d7de !important;
  color: #24292e !important;
}

/* 源码模式 */
:deep(.vditor-sv .CodeMirror) {
  background: #ffffff !important;
  color: #24292e !important;
}

/* 工具栏背景也改为白色 */
:deep(.vditor-toolbar) {
  background: #ffffff !important;
  border-bottom: 1px solid #e1e4e8 !important;
}

/* 历史记录侧边栏 */
.history-sidebar {
  position: fixed;
  right: 0;
  top: 0;
  width: 320px;
  height: 100vh;
  background: var(--color-background);
  border-left: 1px solid var(--color-border);
  box-shadow: -4px 0 12px rgba(0, 0, 0, 0.1);
  display: flex;
  flex-direction: column;
  z-index: 1000;
}

.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px;
  border-bottom: 1px solid var(--color-border);
}

.history-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
}

.btn-close {
  padding: 6px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-close:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.history-item {
  position: relative;
  padding: 12px;
  margin-bottom: 8px;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.history-item:hover {
  border-color: var(--color-primary);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.history-title {
  font-weight: 500;
  color: var(--color-text-primary);
  margin-bottom: 4px;
}

.history-time {
  font-size: 12px;
  color: var(--color-text-tertiary);
  margin-bottom: 6px;
}

.history-summary {
  font-size: 13px;
  color: var(--color-text-secondary);
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
}

.btn-delete {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 4px;
  background: transparent;
  border: none;
  border-radius: 4px;
  color: var(--color-text-tertiary);
  cursor: pointer;
  opacity: 0;
  transition: all 0.2s;
}

.history-item:hover .btn-delete {
  opacity: 1;
}

.btn-delete:hover {
  background: var(--color-danger);
  color: white;
}

/* 导出对话框 */
.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
}

.dialog-content {
  background: var(--color-background);
  border-radius: 12px;
  padding: 24px;
  width: 400px;
  max-width: 90vw;
}

.dialog-content h3 {
  margin: 0 0 20px 0;
  font-size: 18px;
  font-weight: 600;
}

.export-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 20px;
}

.export-btn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text-primary);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.export-btn:hover {
  background: var(--color-primary);
  color: white;
  border-color: var(--color-primary);
}

.btn-cancel {
  width: 100%;
  padding: 10px;
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.btn-cancel:hover {
  background: var(--color-background-secondary);
  color: var(--color-text-primary);
}

/* 过渡动画 */
.slide-right-enter-active,
.slide-right-leave-active {
  transition: transform 0.3s ease;
}

.slide-right-enter-from {
  transform: translateX(100%);
}

.slide-right-leave-to {
  transform: translateX(100%);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
