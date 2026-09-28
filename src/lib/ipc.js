/**
 * 백엔드 호출 단일 진입점.
 * - Tauri 안에서는 Rust 커맨드를 호출한다.
 * - 브라우저(`npm run dev` 단독)에서는 화면 확인용 메모리 mock 으로 대체한다.
 * 컴포넌트는 이 함수를 직접 쓰지 말고 Store 를 통해서만 호출한다.
 */
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { mockInvoke } from './mockBackend'

export const isTauri = typeof window !== 'undefined' && !!window.__TAURI_INTERNALS__

export function invoke(cmd, args) {
  if (isTauri) return tauriInvoke(cmd, args)
  return mockInvoke(cmd, args)
}
