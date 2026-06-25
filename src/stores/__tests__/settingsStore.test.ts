import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useSettingsStore } from '../settingsStore'
import * as api from '../../utils/tauri'

// Mock Tauri API
vi.mock('../../utils/tauri', () => ({
  getSettings: vi.fn(),
  updateSetting: vi.fn(),
  updateHotkey: vi.fn(),
}))

describe('settingsStore', () => {
  beforeEach(() => {
    // 重置 store 状态为默认值
    useSettingsStore.setState({
      settings: {
        retentionDays: 30,
        maxItems: 5000,
        pollIntervalMs: 500,
        hotkey: 'Cmd+Shift+V',
        launchAtLogin: true,
        theme: 'system',
      },
      isLoading: false,
      error: null,
    })
    vi.clearAllMocks()
  })

  describe('初始状态', () => {
    it('应有正确的默认设置', () => {
      const state = useSettingsStore.getState()
      expect(state.settings.retentionDays).toBe(30)
      expect(state.settings.maxItems).toBe(5000)
      expect(state.settings.pollIntervalMs).toBe(500)
      expect(state.settings.hotkey).toBe('Cmd+Shift+V')
      expect(state.settings.launchAtLogin).toBe(true)
      expect(state.settings.theme).toBe('system')
    })

    it('初始 isLoading 应为 false', () => {
      expect(useSettingsStore.getState().isLoading).toBe(false)
    })

    it('初始 error 应为 null', () => {
      expect(useSettingsStore.getState().error).toBeNull()
    })
  })

  describe('fetchSettings', () => {
    it('应能从后端获取设置并解析', async () => {
      vi.mocked(api.getSettings).mockResolvedValue({
        retention_days: '60',
        max_items: '3000',
        poll_interval_ms: '1000',
        hotkey: 'Ctrl+Shift+V',
        launch_at_login: 'true',
        theme: 'dark',
      })

      await useSettingsStore.getState().fetchSettings()

      const state = useSettingsStore.getState()
      expect(state.settings.retentionDays).toBe(60)
      expect(state.settings.maxItems).toBe(3000)
      expect(state.settings.pollIntervalMs).toBe(1000)
      expect(state.settings.hotkey).toBe('Ctrl+Shift+V')
      expect(state.settings.launchAtLogin).toBe(true)
      expect(state.settings.theme).toBe('dark')
      expect(state.isLoading).toBe(false)
      expect(state.error).toBeNull()
    })

    it('获取失败时应设置错误状态', async () => {
      vi.mocked(api.getSettings).mockRejectedValue(new Error('获取失败'))

      await useSettingsStore.getState().fetchSettings()

      const state = useSettingsStore.getState()
      expect(state.isLoading).toBe(false)
      expect(state.error).toBe('Error: 获取失败')
    })

    it('获取过程中 isLoading 应为 true', async () => {
      let resolvePromise: (v: Record<string, string>) => void
      vi.mocked(api.getSettings).mockReturnValue(
        new Promise((resolve) => {
          resolvePromise = resolve
        })
      )

      const fetchPromise = useSettingsStore.getState().fetchSettings()
      expect(useSettingsStore.getState().isLoading).toBe(true)

      resolvePromise!({
        retention_days: '30',
        max_items: '5000',
        poll_interval_ms: '500',
        hotkey: 'Cmd+Shift+V',
        launch_at_login: 'true',
        theme: 'system',
      })
      await fetchPromise

      expect(useSettingsStore.getState().isLoading).toBe(false)
    })

    it('应正确处理 launch_at_login 为 false', async () => {
      vi.mocked(api.getSettings).mockResolvedValue({
        retention_days: '30',
        max_items: '5000',
        poll_interval_ms: '500',
        hotkey: 'Cmd+Shift+V',
        launch_at_login: 'false',
        theme: 'light',
      })

      await useSettingsStore.getState().fetchSettings()

      const state = useSettingsStore.getState()
      expect(state.settings.launchAtLogin).toBe(false)
      expect(state.settings.theme).toBe('light')
    })

    it('应使用默认值处理缺失的字段', async () => {
      vi.mocked(api.getSettings).mockResolvedValue({})

      await useSettingsStore.getState().fetchSettings()

      const state = useSettingsStore.getState()
      expect(state.settings.retentionDays).toBe(30)
      expect(state.settings.maxItems).toBe(5000)
      expect(state.settings.pollIntervalMs).toBe(500)
      expect(state.settings.launchAtLogin).toBe(false)
      expect(state.settings.theme).toBe('system')
    })
  })

  describe('updateSetting', () => {
    it('应能更新单个设置值', async () => {
      vi.mocked(api.updateSetting).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateSetting('retentionDays', 60)

      const state = useSettingsStore.getState()
      expect(state.settings.retentionDays).toBe(60)
      expect(api.updateSetting).toHaveBeenCalledWith('retention_days', '60')
    })

    it('应能更新 theme 设置', async () => {
      vi.mocked(api.updateSetting).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateSetting('theme', 'dark')

      expect(useSettingsStore.getState().settings.theme).toBe('dark')
      expect(api.updateSetting).toHaveBeenCalledWith('theme', 'dark')
    })

    it('应能更新 maxItems 设置', async () => {
      vi.mocked(api.updateSetting).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateSetting('maxItems', 10000)

      expect(useSettingsStore.getState().settings.maxItems).toBe(10000)
      expect(api.updateSetting).toHaveBeenCalledWith('max_items', '10000')
    })

    it('应能更新 launchAtLogin 设置', async () => {
      vi.mocked(api.updateSetting).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateSetting('launchAtLogin', false)

      expect(useSettingsStore.getState().settings.launchAtLogin).toBe(false)
      expect(api.updateSetting).toHaveBeenCalledWith('launch_at_login', 'false')
    })

    it('应能更新 pollIntervalMs 设置', async () => {
      vi.mocked(api.updateSetting).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateSetting('pollIntervalMs', 1000)

      expect(useSettingsStore.getState().settings.pollIntervalMs).toBe(1000)
      expect(api.updateSetting).toHaveBeenCalledWith('poll_interval_ms', '1000')
    })

    it('更新失败时应设置错误状态', async () => {
      vi.mocked(api.updateSetting).mockRejectedValue(new Error('更新失败'))

      await useSettingsStore.getState().updateSetting('retentionDays', 60)

      const state = useSettingsStore.getState()
      expect(state.error).toBe('Error: 更新失败')
      // 本地值应被回滚（因为 set 在 try 块内，失败不会执行）
      expect(state.settings.retentionDays).toBe(30)
    })
  })

  describe('updateHotkey', () => {
    it('应能更新快捷键', async () => {
      vi.mocked(api.updateHotkey).mockResolvedValue(undefined)

      await useSettingsStore.getState().updateHotkey('Ctrl+Shift+V')

      const state = useSettingsStore.getState()
      expect(state.settings.hotkey).toBe('Ctrl+Shift+V')
      expect(state.error).toBeNull()
      expect(api.updateHotkey).toHaveBeenCalledWith('Ctrl+Shift+V')
    })

    it('更新失败时应设置错误状态并抛出异常', async () => {
      vi.mocked(api.updateHotkey).mockRejectedValue(new Error('快捷键更新失败'))

      await expect(
        useSettingsStore.getState().updateHotkey('Invalid+Key')
      ).rejects.toThrow('快捷键更新失败')

      const state = useSettingsStore.getState()
      expect(state.error).toBe('Error: 快捷键更新失败')
    })

    it('更新失败后应清除之前的错误状态', async () => {
      vi.mocked(api.updateHotkey).mockResolvedValue(undefined)

      // 先设置一个错误
      useSettingsStore.setState({ error: '之前的错误' })

      await useSettingsStore.getState().updateHotkey('Cmd+Shift+Z')

      expect(useSettingsStore.getState().error).toBeNull()
    })
  })
})
