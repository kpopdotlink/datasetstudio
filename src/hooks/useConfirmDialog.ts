import { useState, useCallback } from 'react';

export interface ConfirmOptions {
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  variant?: 'danger' | 'warning' | 'info';
}

export function useConfirmDialog() {
  const [isOpen, setIsOpen] = useState(false);
  const [options, setOptions] = useState<ConfirmOptions>({
    title: '',
    message: '',
  });
  const [resolveCallback, setResolveCallback] = useState<
    ((value: boolean) => void) | null
  >(null);

  const confirm = useCallback((opts: ConfirmOptions): Promise<boolean> => {
    setOptions(opts);
    setIsOpen(true);

    return new Promise((resolve) => {
      setResolveCallback(() => resolve);
    });
  }, []);

  const handleConfirm = useCallback(() => {
    if (resolveCallback) {
      resolveCallback(true);
    }
    setIsOpen(false);
    setResolveCallback(null);
  }, [resolveCallback]);

  const handleClose = useCallback(() => {
    if (resolveCallback) {
      resolveCallback(false);
    }
    setIsOpen(false);
    setResolveCallback(null);
  }, [resolveCallback]);

  return {
    isOpen,
    options,
    confirm,
    handleConfirm,
    handleClose,
  };
}

// 사전 정의된 확인 옵션
export const confirmPresets = {
  delete: (itemName: string): ConfirmOptions => ({
    title: '삭제 확인',
    message: `"${itemName}"을(를) 삭제하시겠습니까? 이 작업은 되돌릴 수 없습니다.`,
    confirmText: '삭제',
    cancelText: '취소',
    variant: 'danger',
  }),

  overwrite: (itemName: string): ConfirmOptions => ({
    title: '덮어쓰기 확인',
    message: `"${itemName}"이(가) 이미 존재합니다. 덮어쓰시겠습니까?`,
    confirmText: '덮어쓰기',
    cancelText: '취소',
    variant: 'warning',
  }),

  discard: (): ConfirmOptions => ({
    title: '변경사항 폐기',
    message:
      '저장하지 않은 변경사항이 있습니다. 정말로 나가시겠습니까?',
    confirmText: '나가기',
    cancelText: '계속 편집',
    variant: 'warning',
  }),

  rechunk: (): ConfirmOptions => ({
    title: '청크 재생성',
    message:
      '기존 청크가 모두 삭제되고 새로 생성됩니다. 계속하시겠습니까?',
    confirmText: '재생성',
    cancelText: '취소',
    variant: 'warning',
  }),
};
