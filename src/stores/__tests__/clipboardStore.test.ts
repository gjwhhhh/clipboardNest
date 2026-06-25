import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useClipboardStore } from '../clipboardStore'
import * as api from '../../utils/tauri'
import type { ClipboardItem } from '../../types'

// Mock Tauri API
vi.mock('../../utils/tauri', () => ({
  getClipboardHistory: vi.fn(),
  searchClipboard: vi.fn(),
  copyToClipboard: vi.fn(),
  deleteClipboardItem: vi.fn(),
  pinClipboardItem: vi.fn(),
  favoriteClipboardItem: vi.fn(),
  clearAllHistory: vi.fn(),
}))

const mockItem: ClipboardItem = {
  id: 1,
  contentType: 'text',
  content: '测试内容',
  preview: '测试内容',
  contentHash: 'abc123',
  fileName: null,
  fileSize: null,
  filePath: null,
  thumbnailPath: null,
  sourceApp: 'TestApp',
  isPinned: false,
  isFavorite: false,
  createdAt: '2026-01-01T00:00:00Z',
  updatedAt: '2026-01-01T00:00:00Z',
}

describe('clipboardStore', () => {
  beforeEach(() => {
    // 重置 store 状态
    useClipboardStore.setState({
      items: [],
      filteredItems: [],
      selectedItem: null,
      hoveredItem: null,
      searchQuery: '',
      activeFilter: 'all',
      isLoading: false,
      error: null,
      toast: null,
    })
    vi.clearAllMocks()
  })

  describe('初始状态', () => {
    it('初始状态应为空', () => {
      const state = useClipboardStore.getState()
      expect(state.items).toEqual([])
      expect(state.filteredItems).toEqual([])
      expect(state.selectedItem).toBeNull()
      expect(state.hoveredItem).toBeNull()
      expect(state.searchQuery).toBe('')
      expect(state.activeFilter).toBe('all')
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
      expect(state.toast).toBeNull()
    })
  })

  describe('fetchItems', () => {
    it('应能获取剪切板历史', async () => {
      const mockItems = [mockItem]
      vi.mocked(api.getClipboardHistory).mockResolvedValue(mockItems)

      await useClipboardStore.getState().fetchItems()

      const state = useClipboardStore.getState()
      expect(state.items).toEqual(mockItems)
      expect(state.filteredItems).toEqual(mockItems)
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })

    it('获取失败时应设置错误状态', async () => {
      vi.mocked(api.getClipboardHistory).mockRejectedValue(new Error('获取失败'))

      await useClipboardStore.getState().fetchItems()

      const state = useClipboardStore.getState()
      expect(state.isLoading).toBe(false)
      expect(state.error).toBe('Error: 获取失败')
    })

    it('应根据 filter 类型传递参数', async () => {
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      useClipboardStore.setState({ activeFilter: 'text' })
      await useClipboardStore.getState().fetchItems()

      expect(api.getClipboardHistory).toHaveBeenCalledWith('text')
    })

    it('filter 为 all 时应传递 undefined', async () => {
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      useClipboardStore.setState({ activeFilter: 'all' })
      await useClipboardStore.getState().fetchItems()

      expect(api.getClipboardHistory).toHaveBeenCalledWith(undefined)
    })
  })

  describe('search', () => {
    it('应能搜索剪切板内容', async () => {
      const mockItems = [mockItem]
      vi.mocked(api.searchClipboard).mockResolvedValue(mockItems)

      await useClipboardStore.getState().search('测试')

      const state = useClipboardStore.getState()
      expect(state.searchQuery).toBe('测试')
      expect(state.filteredItems).toEqual(mockItems)
      expect(state.isLoading).toBe(false)
    })

    it('搜索空字符串时应调用 fetchItems', async () => {
      vi.mocked(api.getClipboardHistory).mockResolvedValue([mockItem])

      await useClipboardStore.getState().search('')

      expect(api.getClipboardHistory).toHaveBeenCalled()
      expect(api.searchClipboard).not.toHaveBeenCalled()
    })

    it('搜索失败时应设置错误状态', async () => {
      vi.mocked(api.searchClipboard).mockRejectedValue(new Error('搜索失败'))

      await useClipboardStore.getState().search('测试')

      const state = useClipboardStore.getState()
      expect(state.isLoading).toBe(false)
      expect(state.error).toBe('Error: 搜索失败')
    })
  })

  describe('setFilter', () => {
    it('应能设置过滤类型并刷新列表', async () => {
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().setFilter('image')

      const state = useClipboardStore.getState()
      expect(state.activeFilter).toBe('image')
      expect(api.getClipboardHistory).toHaveBeenCalledWith('image')
    })
  })

  describe('selectItem', () => {
    it('应能设置选中项', () => {
      useClipboardStore.getState().selectItem(mockItem)
      expect(useClipboardStore.getState().selectedItem).toEqual(mockItem)
    })

    it('应能清除选中项', () => {
      useClipboardStore.setState({ selectedItem: mockItem })
      useClipboardStore.getState().selectItem(null)
      expect(useClipboardStore.getState().selectedItem).toBeNull()
    })
  })

  describe('setHoveredItem', () => {
    it('应能设置悬停项', () => {
      useClipboardStore.getState().setHoveredItem(mockItem)
      expect(useClipboardStore.getState().hoveredItem).toEqual(mockItem)
    })

    it('应能清除悬停项', () => {
      useClipboardStore.setState({ hoveredItem: mockItem })
      useClipboardStore.getState().setHoveredItem(null)
      expect(useClipboardStore.getState().hoveredItem).toBeNull()
    })
  })

  describe('copyItem', () => {
    it('应能复制剪切板项目', async () => {
      vi.mocked(api.copyToClipboard).mockResolvedValue(undefined)

      await useClipboardStore.getState().copyItem(mockItem)

      expect(api.copyToClipboard).toHaveBeenCalledWith(mockItem.id)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已复制到剪切板' })
      )
    })

    it('复制失败时应显示错误 toast', async () => {
      vi.mocked(api.copyToClipboard).mockRejectedValue(new Error('复制失败'))

      await expect(useClipboardStore.getState().copyItem(mockItem)).rejects.toThrow()

      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'error', message: '复制失败' })
      )
    })
  })

  describe('deleteItem', () => {
    it('应能删除剪切板项目', async () => {
      vi.mocked(api.deleteClipboardItem).mockResolvedValue(undefined)
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().deleteItem(mockItem.id)

      expect(api.deleteClipboardItem).toHaveBeenCalledWith(mockItem.id)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已删除' })
      )
    })

    it('删除失败时应显示错误 toast', async () => {
      vi.mocked(api.deleteClipboardItem).mockRejectedValue(new Error('删除失败'))

      await useClipboardStore.getState().deleteItem(mockItem.id)

      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'error', message: '删除失败' })
      )
    })
  })

  describe('pinItem', () => {
    it('应能置顶项目', async () => {
      vi.mocked(api.pinClipboardItem).mockResolvedValue(undefined)
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().pinItem(mockItem.id, true)

      expect(api.pinClipboardItem).toHaveBeenCalledWith(mockItem.id, true)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已置顶' })
      )
    })

    it('应能取消置顶', async () => {
      vi.mocked(api.pinClipboardItem).mockResolvedValue(undefined)
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().pinItem(mockItem.id, false)

      expect(api.pinClipboardItem).toHaveBeenCalledWith(mockItem.id, false)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已取消置顶' })
      )
    })

    it('置顶失败时应显示错误 toast', async () => {
      vi.mocked(api.pinClipboardItem).mockRejectedValue(new Error('置顶失败'))

      await useClipboardStore.getState().pinItem(mockItem.id, true)

      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'error', message: '置顶失败' })
      )
    })
  })

  describe('favoriteItem', () => {
    it('应能收藏项目', async () => {
      vi.mocked(api.favoriteClipboardItem).mockResolvedValue(undefined)
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().favoriteItem(mockItem.id, true)

      expect(api.favoriteClipboardItem).toHaveBeenCalledWith(mockItem.id, true)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已收藏' })
      )
    })

    it('应能取消收藏', async () => {
      vi.mocked(api.favoriteClipboardItem).mockResolvedValue(undefined)
      vi.mocked(api.getClipboardHistory).mockResolvedValue([])

      await useClipboardStore.getState().favoriteItem(mockItem.id, false)

      expect(api.favoriteClipboardItem).toHaveBeenCalledWith(mockItem.id, false)
      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已取消收藏' })
      )
    })

    it('收藏失败时应显示错误 toast', async () => {
      vi.mocked(api.favoriteClipboardItem).mockRejectedValue(new Error('收藏失败'))

      await useClipboardStore.getState().favoriteItem(mockItem.id, true)

      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'error', message: '收藏失败' })
      )
    })
  })

  describe('clearAll', () => {
    it('应能清除所有历史', async () => {
      vi.mocked(api.clearAllHistory).mockResolvedValue(undefined)
      useClipboardStore.setState({ items: [mockItem], filteredItems: [mockItem], selectedItem: mockItem })

      await useClipboardStore.getState().clearAll()

      expect(api.clearAllHistory).toHaveBeenCalled()
      const state = useClipboardStore.getState()
      expect(state.items).toEqual([])
      expect(state.filteredItems).toEqual([])
      expect(state.selectedItem).toBeNull()
      expect(state.toast).toEqual(
        expect.objectContaining({ type: 'success', message: '已清除所有历史' })
      )
    })

    it('清除失败时应显示错误 toast', async () => {
      vi.mocked(api.clearAllHistory).mockRejectedValue(new Error('清除失败'))

      await useClipboardStore.getState().clearAll()

      expect(useClipboardStore.getState().toast).toEqual(
        expect.objectContaining({ type: 'error', message: '清除失败' })
      )
    })
  })

  describe('showToast / hideToast', () => {
    it('应能显示成功 toast', () => {
      useClipboardStore.getState().showToast('success', '操作成功')
      const state = useClipboardStore.getState()
      expect(state.toast).toEqual(
        expect.objectContaining({ type: 'success', message: '操作成功' })
      )
    })

    it('应能显示错误 toast', () => {
      useClipboardStore.getState().showToast('error', '操作失败')
      const state = useClipboardStore.getState()
      expect(state.toast).toEqual(
        expect.objectContaining({ type: 'error', message: '操作失败' })
      )
    })

    it('toast 应有唯一 id', () => {
      useClipboardStore.getState().showToast('success', '消息1')
      const toast1 = useClipboardStore.getState().toast
      useClipboardStore.getState().showToast('success', '消息2')
      const toast2 = useClipboardStore.getState().toast
      expect(toast1?.id).not.toBe(toast2?.id)
    })

    it('应能隐藏 toast', () => {
      useClipboardStore.getState().showToast('success', '操作成功')
      useClipboardStore.getState().hideToast()
      expect(useClipboardStore.getState().toast).toBeNull()
    })
  })
})
