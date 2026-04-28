import "@testing-library/jest-dom";
import { vi } from "vitest";

// Tiptap editor requires some browser APIs not present in jsdom
// Mock minimal required APIs for component mount tests
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Selection API mock for ProseMirror
window.getSelection = () =>
  ({
    removeAllRanges: () => {},
    addRange: () => {},
    getRangeAt: () => ({ collapse: () => {} }),
    rangeCount: 0,
    anchorNode: null,
    anchorOffset: 0,
    focusNode: null,
    focusOffset: 0,
    isCollapsed: true,
    toString: () => "",
  } as unknown as Selection);

// document.caretPositionFromPoint mock
// @ts-expect-error caretPositionFromPoint is not in standard types
document.caretPositionFromPoint = () => null;
