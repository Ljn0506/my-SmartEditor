import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import TiptapEditor from "./TiptapEditor";

describe("TiptapEditor", () => {
  it("renders editor toolbar buttons", () => {
    render(<TiptapEditor />);
    // Toolbar buttons have title attributes
    expect(screen.getByTitle("加粗")).toBeInTheDocument();
    expect(screen.getByTitle("斜体")).toBeInTheDocument();
    expect(screen.getByTitle("一级标题")).toBeInTheDocument();
    expect(screen.getByTitle("二级标题")).toBeInTheDocument();
    expect(screen.getByTitle("无序列表")).toBeInTheDocument();
    expect(screen.getByTitle("有序列表")).toBeInTheDocument();
    expect(screen.getByTitle("引用")).toBeInTheDocument();
    expect(screen.getByTitle("撤销")).toBeInTheDocument();
    expect(screen.getByTitle("重做")).toBeInTheDocument();
  });

  it("calls onEditorReady when editor is initialized", async () => {
    const onEditorReady = vi.fn();
    render(<TiptapEditor onEditorReady={onEditorReady} />);
    // Editor initialization is async; wait briefly then verify callback was called
    await new Promise((r) => setTimeout(r, 100));
    expect(onEditorReady).toHaveBeenCalled();
  });

  it("renders editor content area", () => {
    render(<TiptapEditor />);
    // ProseMirror editor content div should exist
    const editorContent = document.querySelector(".ProseMirror");
    expect(editorContent).toBeInTheDocument();
  });
});
