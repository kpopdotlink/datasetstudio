import { useState } from "react";
import {
  checkDocumentChanges,
  resolveDocumentChange,
  DocumentChangeInfo,
  ChangeDetectionResult,
} from "../../api/source";

interface ChangeDetectionProps {
  onComplete?: () => void;
}

export function ChangeDetection({ onComplete }: ChangeDetectionProps) {
  const [loading, setLoading] = useState(false);
  const [checking, setChecking] = useState(false);
  const [result, setResult] = useState<ChangeDetectionResult | null>(null);
  const [processingId, setProcessingId] = useState<number | null>(null);

  const handleCheck = async () => {
    setChecking(true);
    try {
      const res = await checkDocumentChanges();
      setResult(res);
    } catch (err) {
      console.error("Failed to check changes:", err);
    } finally {
      setChecking(false);
    }
  };

  const handleResolve = async (
    change: DocumentChangeInfo,
    action: "rechunk" | "update_checksum" | "remove"
  ) => {
    setProcessingId(change.document_id);
    try {
      await resolveDocumentChange(change.document_id, action);
      // 목록에서 제거
      setResult((prev) =>
        prev
          ? {
              ...prev,
              changes: prev.changes.filter(
                (c) => c.document_id !== change.document_id
              ),
            }
          : null
      );
      if (onComplete) onComplete();
    } catch (err) {
      console.error("Failed to resolve change:", err);
    } finally {
      setProcessingId(null);
    }
  };

  const handleResolveAll = async (
    action: "rechunk" | "update_checksum" | "remove"
  ) => {
    if (!result) return;
    setLoading(true);
    try {
      for (const change of result.changes) {
        await resolveDocumentChange(change.document_id, action);
      }
      setResult({ ...result, changes: [] });
      if (onComplete) onComplete();
    } catch (err) {
      console.error("Failed to resolve all changes:", err);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-medium">문서 변경 감지</h3>
        <button
          onClick={handleCheck}
          disabled={checking}
          className="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50"
        >
          {checking ? "확인 중..." : "변경 확인"}
        </button>
      </div>

      {result && (
        <div className="space-y-3">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            {result.total_checked}개 문서 확인됨, {result.changes.length}개 변경
            감지
          </p>

          {result.changes.length > 0 && (
            <>
              <div className="flex gap-2">
                <button
                  onClick={() => handleResolveAll("rechunk")}
                  disabled={loading}
                  className="px-2 py-1 text-xs bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
                >
                  모두 재청크
                </button>
                <button
                  onClick={() => handleResolveAll("update_checksum")}
                  disabled={loading}
                  className="px-2 py-1 text-xs bg-gray-600 text-white rounded hover:bg-gray-700 disabled:opacity-50"
                >
                  모두 무시
                </button>
              </div>

              <div className="border rounded-lg divide-y dark:border-gray-700 dark:divide-gray-700">
                {result.changes.map((change) => (
                  <div
                    key={change.document_id}
                    className="p-3 flex items-center justify-between"
                  >
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span
                          className={`px-1.5 py-0.5 text-xs rounded ${
                            change.change_type === "deleted"
                              ? "bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300"
                              : "bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300"
                          }`}
                        >
                          {change.change_type === "deleted" ? "삭제됨" : "변경됨"}
                        </span>
                        <span className="font-medium truncate">
                          {change.display_name}
                        </span>
                      </div>
                      <p className="text-xs text-gray-500 truncate mt-0.5">
                        {change.original_path}
                      </p>
                    </div>
                    <div className="flex gap-1 ml-2">
                      {change.change_type === "modified" && (
                        <>
                          <button
                            onClick={() => handleResolve(change, "rechunk")}
                            disabled={processingId === change.document_id}
                            className="px-2 py-1 text-xs bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
                            title="기존 청크 삭제 후 다시 처리"
                          >
                            재청크
                          </button>
                          <button
                            onClick={() =>
                              handleResolve(change, "update_checksum")
                            }
                            disabled={processingId === change.document_id}
                            className="px-2 py-1 text-xs bg-gray-500 text-white rounded hover:bg-gray-600 disabled:opacity-50"
                            title="변경사항 무시"
                          >
                            무시
                          </button>
                        </>
                      )}
                      <button
                        onClick={() => handleResolve(change, "remove")}
                        disabled={processingId === change.document_id}
                        className="px-2 py-1 text-xs bg-red-600 text-white rounded hover:bg-red-700 disabled:opacity-50"
                        title="문서 삭제"
                      >
                        삭제
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}

          {result.changes.length === 0 && (
            <div className="p-4 text-center text-green-600 bg-green-50 dark:bg-green-900/20 dark:text-green-400 rounded-lg">
              모든 문서가 최신 상태입니다
            </div>
          )}
        </div>
      )}

      {!result && (
        <p className="text-sm text-gray-500 dark:text-gray-400">
          "변경 확인" 버튼을 클릭하여 소스 파일의 변경사항을 확인하세요.
        </p>
      )}
    </div>
  );
}
