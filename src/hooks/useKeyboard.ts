import { useEffect, useCallback } from "react";

type KeyHandler = () => void;
type KeyMap = Record<string, KeyHandler>;

export function useKeyboard(keyMap: KeyMap) {
  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      // 입력 필드에서는 무시
      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        return;
      }

      // Modifier 키와 함께 누르면 무시 (Cmd+Z 등)
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      const key = event.key;
      const handler = keyMap[key];

      if (handler) {
        event.preventDefault();
        handler();
      }
    },
    [keyMap]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}
