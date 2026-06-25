import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useKeyboard } from '../useKeyboard'

describe('useKeyboard', () => {
  const mockHandlers = {
    onEnter: vi.fn(),
    onEscape: vi.fn(),
    onArrowUp: vi.fn(),
    onArrowDown: vi.fn(),
    onDelete: vi.fn(),
  }

  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('应注册 keydown 事件监听器', () => {
    const addSpy = vi.spyOn(window, 'addEventListener')
    const removeSpy = vi.spyOn(window, 'removeEventListener')

    const { unmount } = renderHook(() => useKeyboard(mockHandlers))

    expect(addSpy).toHaveBeenCalledWith('keydown', expect.any(Function))

    unmount()

    expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function))

    addSpy.mockRestore()
    removeSpy.mockRestore()
  })

  describe('Enter 键', () => {
    it('非输入框中按下 Enter 应调用 onEnter', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'Enter' })
      window.dispatchEvent(event)

      expect(mockHandlers.onEnter).toHaveBeenCalledTimes(1)
    })

    it('在输入框中按下 Enter 不应调用 onEnter', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const input = document.createElement('input')
      document.body.appendChild(input)

      const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })
      input.dispatchEvent(event)

      expect(mockHandlers.onEnter).not.toHaveBeenCalled()

      document.body.removeChild(input)
    })

    it('在 textarea 中按下 Enter 不应调用 onEnter', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const textarea = document.createElement('textarea')
      document.body.appendChild(textarea)

      const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })
      textarea.dispatchEvent(event)

      expect(mockHandlers.onEnter).not.toHaveBeenCalled()

      document.body.removeChild(textarea)
    })

    it('在 contentEditable 元素中按下 Enter 不应调用 onEnter', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const div = document.createElement('div')
      div.contentEditable = 'true'
      document.body.appendChild(div)

      const event = new KeyboardEvent('keydown', { key: 'Enter', bubbles: true })
      div.dispatchEvent(event)

      expect(mockHandlers.onEnter).not.toHaveBeenCalled()

      document.body.removeChild(div)
    })
  })

  describe('Escape 键', () => {
    it('按下 Escape 应调用 onEscape', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'Escape' })
      window.dispatchEvent(event)

      expect(mockHandlers.onEscape).toHaveBeenCalledTimes(1)
    })

    it('在输入框中按下 Escape 仍应调用 onEscape', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const input = document.createElement('input')
      document.body.appendChild(input)

      const event = new KeyboardEvent('keydown', { key: 'Escape', bubbles: true })
      input.dispatchEvent(event)

      expect(mockHandlers.onEscape).toHaveBeenCalledTimes(1)

      document.body.removeChild(input)
    })
  })

  describe('ArrowUp 键', () => {
    it('非输入框中按下 ArrowUp 应调用 onArrowUp', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'ArrowUp' })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
      window.dispatchEvent(event)

      expect(mockHandlers.onArrowUp).toHaveBeenCalledTimes(1)
      expect(preventDefaultSpy).toHaveBeenCalled()

      preventDefaultSpy.mockRestore()
    })

    it('在输入框中按下 ArrowUp 不应调用 onArrowUp', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const input = document.createElement('input')
      document.body.appendChild(input)

      const event = new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true })
      input.dispatchEvent(event)

      expect(mockHandlers.onArrowUp).not.toHaveBeenCalled()

      document.body.removeChild(input)
    })
  })

  describe('ArrowDown 键', () => {
    it('非输入框中按下 ArrowDown 应调用 onArrowDown', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'ArrowDown' })
      const preventDefaultSpy = vi.spyOn(event, 'preventDefault')
      window.dispatchEvent(event)

      expect(mockHandlers.onArrowDown).toHaveBeenCalledTimes(1)
      expect(preventDefaultSpy).toHaveBeenCalled()

      preventDefaultSpy.mockRestore()
    })

    it('在输入框中按下 ArrowDown 不应调用 onArrowDown', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const input = document.createElement('input')
      document.body.appendChild(input)

      const event = new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true })
      input.dispatchEvent(event)

      expect(mockHandlers.onArrowDown).not.toHaveBeenCalled()

      document.body.removeChild(input)
    })
  })

  describe('Delete / Backspace 键', () => {
    it('按下 Cmd+Delete 应调用 onDelete', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'Delete', metaKey: true })
      window.dispatchEvent(event)

      expect(mockHandlers.onDelete).toHaveBeenCalledTimes(1)
    })

    it('按下 Cmd+Backspace 应调用 onDelete', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'Backspace', metaKey: true })
      window.dispatchEvent(event)

      expect(mockHandlers.onDelete).toHaveBeenCalledTimes(1)
    })

    it('不按 metaKey 时按下 Delete 不应调用 onDelete', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const event = new KeyboardEvent('keydown', { key: 'Delete' })
      window.dispatchEvent(event)

      expect(mockHandlers.onDelete).not.toHaveBeenCalled()
    })

    it('在输入框中按下 Cmd+Delete 不应调用 onDelete', () => {
      renderHook(() => useKeyboard(mockHandlers))

      const input = document.createElement('input')
      document.body.appendChild(input)

      const event = new KeyboardEvent('keydown', { key: 'Delete', metaKey: true, bubbles: true })
      input.dispatchEvent(event)

      expect(mockHandlers.onDelete).not.toHaveBeenCalled()

      document.body.removeChild(input)
    })
  })

  describe('事件监听器清理', () => {
    it('卸载后应移除事件监听器', () => {
      const removeSpy = vi.spyOn(window, 'removeEventListener')

      const { unmount } = renderHook(() => useKeyboard(mockHandlers))
      unmount()

      expect(removeSpy).toHaveBeenCalledWith('keydown', expect.any(Function))

      removeSpy.mockRestore()
    })
  })

  describe('处理器更新', () => {
    it('处理器变更后应使用新的处理器', () => {
      const handlers1 = { onEnter: vi.fn() }
      const handlers2 = { onEnter: vi.fn() }

      const { rerender } = renderHook(
        ({ handlers }) => useKeyboard(handlers),
        { initialProps: { handlers: handlers1 } }
      )

      // 第一次渲染使用 handlers1
      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
      expect(handlers1.onEnter).toHaveBeenCalledTimes(1)
      expect(handlers2.onEnter).not.toHaveBeenCalled()

      // 更新渲染使用 handlers2
      rerender({ handlers: handlers2 })

      window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter' }))
      expect(handlers1.onEnter).toHaveBeenCalledTimes(1) // 不再调用
      expect(handlers2.onEnter).toHaveBeenCalledTimes(1)
    })
  })
})
