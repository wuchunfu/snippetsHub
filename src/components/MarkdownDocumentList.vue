<template>
  <div class="markdown-list">
    <div class="list-header">
      <div class="header-left">
        <div class="title-section">
          <h2>Markdown 文档</h2>
          <div class="statistics">
            <span class="stat-badge">{{ documents.length }} 个文档</span>
          </div>
        </div>
      </div>
      <div class="header-right">
        <button @click="createNewDocument" class="btn-create">
          <Plus :size="16" />
          新建文档
        </button>
      </div>
    </div>

    <div class="search-filter-bar">
      <div class="search-section">
        <div class="search-box">
          <Search :size="18" class="search-icon" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="搜索文档标题、标签或内容..."
            class="search-input"
          />
          <div class="search-actions">
            <button 
              v-if="searchQuery"
              @click="searchQuery = ''"
              class="clear-btn"
              title="清除搜索"
            >
              <X :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="documents-grid">
      <div
        v-for="doc in filteredDocuments"
        :key="doc.id"
        class="document-card"
        @click="openDocument(doc.id)"
      >
        <div class="card-header">
          <div class="card-title-section">
            <FileText :size="20" class="doc-icon" />
            <h3 class="card-title">{{ doc.title || '无标题文档' }}</h3>
          </div>
          <div class="card-actions" @click.stop>
            <button
              @click="renameDoc(doc)"
              class="action-btn"
              title="重命名"
            >
              <Edit2 :size="14" />
            </button>
            <button
              @click="deleteDoc(doc.id)"
              class="action-btn delete"
              title="删除"
            >
              <Trash2 :size="14" />
            </button>
          </div>
        </div>

        <div class="card-preview" v-if="doc.content">
          {{ getPreviewText(doc.content) }}
        </div>
        <div class="card-preview empty" v-else>
          空文档
        </div>

        <div class="card-tags" v-if="doc.tags && doc.tags.length > 0">
          <span
            v-for="tag in doc.tags.slice(0, 3)"
            :key="tag"
            class="tag"
          >
            {{ tag }}
          </span>
          <span v-if="doc.tags.length > 3" class="tag more">
            +{{ doc.tags.length - 3 }}
          </span>
        </div>

        <div class="card-footer">
          <div class="card-stats">
            <span class="stat-item">
              <Type :size="12" />
              {{ getWordCount(doc.content) }} 字
            </span>
            <span class="stat-item">
              <Clock :size="12" />
              {{ formatDate(doc.modifiedAt) }}
            </span>
          </div>
        </div>
      </div>

      <div v-if="filteredDocuments.length === 0" class="empty-state">
        <FileText :size="48" class="empty-icon" />
        <p class="empty-title">{{ searchQuery ? '未找到匹配的文档' : '还没有 Markdown 文档' }}</p>
        <p class="empty-description">
          {{ searchQuery ? '尝试其他搜索词' : '点击上方"新建文档"按钮创建第一个文档' }}
        </p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { useMarkdownStore } from '../stores/markdownStore'
import { Plus, Search, X, FileText, Edit2, Trash2, Type, Clock } from 'lucide-vue-next'

const emit = defineEmits(['open-document', 'create-document'])

const markdownStore = useMarkdownStore()
const searchQuery = ref('')

const documents = computed(() => markdownStore.documents)

const filteredDocuments = computed(() => {
  if (!searchQuery.value) {
    return documents.value
  }
  
  const query = searchQuery.value.toLowerCase()
  return documents.value.filter(doc => {
    const titleMatch = (doc.title || '').toLowerCase().includes(query)
    const contentMatch = (doc.content || '').toLowerCase().includes(query)
    const tagsMatch = (doc.tags || []).some(tag => tag.toLowerCase().includes(query))
    return titleMatch || contentMatch || tagsMatch
  })
})

const createNewDocument = () => {
  const newDoc = markdownStore.createDocument('新建文档')
  emit('open-document', newDoc.id)
}

const openDocument = (docId) => {
  emit('open-document', docId)
}

const renameDoc = (doc) => {
  const newTitle = prompt('输入新标题:', doc.title)
  if (newTitle && newTitle.trim()) {
    markdownStore.renameDocument(doc.id, newTitle.trim())
  }
}

const deleteDoc = (docId) => {
  if (confirm('确定要删除这个文档吗？此操作无法撤销。')) {
    markdownStore.deleteDocument(docId)
  }
}

const getPreviewText = (content) => {
  if (!content) return ''
  const text = content.replace(/[#*`_~\[\]()]/g, '').trim()
  return text.length > 120 ? text.substring(0, 120) + '...' : text
}

const getWordCount = (content) => {
  if (!content) return 0
  return content.length
}

const formatDate = (dateStr) => {
  if (!dateStr) return ''
  const date = new Date(dateStr)
  const now = new Date()
  const diff = now - date
  
  const seconds = Math.floor(diff / 1000)
  const minutes = Math.floor(seconds / 60)
  const hours = Math.floor(minutes / 60)
  const days = Math.floor(hours / 24)
  
  if (seconds < 60) return '刚刚'
  if (minutes < 60) return `${minutes} 分钟前`
  if (hours < 24) return `${hours} 小时前`
  if (days < 7) return `${days} 天前`
  
  return date.toLocaleDateString('zh-CN')
}
</script>

<style scoped>
.markdown-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-background);
}

.list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-background-secondary);
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.title-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.title-section h2 {
  font-size: 20px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
}

.statistics {
  display: flex;
  gap: 8px;
}

.stat-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: var(--color-background-tertiary);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
  color: var(--color-text-secondary);
}

.header-right {
  display: flex;
  gap: 12px;
}

.btn-create {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--color-primary);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-create:hover {
  background: var(--color-primary-hover);
  transform: translateY(-1px);
}

.search-filter-bar {
  padding: 16px 24px;
  border-bottom: 1px solid var(--color-border);
  background: var(--color-background-secondary);
}

.search-section {
  max-width: 600px;
}

.search-box {
  position: relative;
  display: flex;
  align-items: center;
}

.search-icon {
  position: absolute;
  left: 12px;
  color: var(--color-text-tertiary);
}

.search-input {
  flex: 1;
  padding: 10px 40px 10px 42px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 8px;
  color: var(--color-text-primary);
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s ease;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.search-actions {
  position: absolute;
  right: 8px;
  display: flex;
  gap: 4px;
}

.clear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  background: var(--color-background-secondary);
  border: none;
  border-radius: 4px;
  color: var(--color-text-tertiary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.clear-btn:hover {
  background: var(--color-border);
  color: var(--color-text-primary);
}

.documents-grid {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 20px;
  align-content: start;
}

.document-card {
  background: var(--color-background-secondary);
  border: 1px solid var(--color-border);
  border-radius: 12px;
  padding: 16px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.document-card:hover {
  border-color: var(--color-primary);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--color-shadow);
}

.card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 8px;
}

.card-title-section {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.doc-icon {
  color: var(--color-primary);
  flex-shrink: 0;
}

.card-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.card-actions {
  display: flex;
  gap: 4px;
  opacity: 0;
  transition: opacity 0.2s ease;
}

.document-card:hover .card-actions {
  opacity: 1;
}

.action-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: var(--color-background);
  border: 1px solid var(--color-border);
  border-radius: 6px;
  color: var(--color-text-secondary);
  cursor: pointer;
  transition: all 0.2s ease;
}

.action-btn:hover {
  background: var(--color-border);
  color: var(--color-text-primary);
}

.action-btn.delete:hover {
  background: var(--color-error);
  border-color: var(--color-error);
  color: white;
}

.card-preview {
  font-size: 13px;
  line-height: 1.6;
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  min-height: 60px;
}

.card-preview.empty {
  color: var(--color-text-tertiary);
  font-style: italic;
}

.card-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tag {
  padding: 3px 8px;
  background: var(--color-background-tertiary);
  border: 1px solid var(--color-border);
  border-radius: 4px;
  font-size: 11px;
  color: var(--color-text-secondary);
}

.tag.more {
  background: transparent;
  border-style: dashed;
}

.card-footer {
  padding-top: 12px;
  border-top: 1px solid var(--color-border);
}

.card-stats {
  display: flex;
  gap: 16px;
}

.stat-item {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 12px;
  color: var(--color-text-tertiary);
}

.empty-state {
  grid-column: 1 / -1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 80px 20px;
  text-align: center;
}

.empty-icon {
  color: var(--color-text-tertiary);
  margin-bottom: 16px;
  opacity: 0.5;
}

.empty-title {
  font-size: 18px;
  font-weight: 600;
  color: var(--color-text-primary);
  margin: 0 0 8px 0;
}

.empty-description {
  font-size: 14px;
  color: var(--color-text-secondary);
  margin: 0;
}
</style>
