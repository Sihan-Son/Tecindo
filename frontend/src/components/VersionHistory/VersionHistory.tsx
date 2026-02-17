import { useEffect, useState } from 'react';
import { diffLines } from 'diff';
import { fetchDocumentVersions, fetchVersionContent } from '@/api/client';
import type { DocumentVersionSummary } from '@/lib/types';

interface Props {
  documentId: string;
  currentContent: string;
  onClose: () => void;
}

export function VersionHistory({ documentId, currentContent, onClose }: Props) {
  const [versions, setVersions] = useState<DocumentVersionSummary[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [selectedContent, setSelectedContent] = useState<string | null>(null);
  const [compareContent, setCompareContent] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    setLoading(true);
    fetchDocumentVersions(documentId)
      .then((v) => {
        setVersions(v);
        if (v.length > 0) {
          setSelectedId(v[0].id);
        }
      })
      .catch(console.error)
      .finally(() => setLoading(false));
  }, [documentId]);

  useEffect(() => {
    if (!selectedId) {
      setSelectedContent(null);
      setCompareContent(null);
      return;
    }

    fetchVersionContent(selectedId)
      .then((v) => {
        setSelectedContent(v.content);

        // 이전 버전 찾기
        const idx = versions.findIndex((ver) => ver.id === selectedId);
        if (idx < versions.length - 1) {
          // 이전 버전이 있으면 그것과 비교
          fetchVersionContent(versions[idx + 1].id)
            .then((prev) => setCompareContent(prev.content))
            .catch(() => setCompareContent(null));
        } else {
          // 가장 오래된 버전이면 빈 문자열과 비교
          setCompareContent('');
        }
      })
      .catch(console.error);
  }, [selectedId, versions]);

  const selectedVersion = versions.find((v) => v.id === selectedId);

  const diffResult =
    selectedContent !== null && compareContent !== null
      ? diffLines(compareContent, selectedContent)
      : [];

  return (
    <div className="version-history-panel">
      <div className="version-history-header">
        <h3>버전 기록</h3>
        <button className="version-close-btn" onClick={onClose} title="닫기">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4L12 12M12 4L4 12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
        </button>
      </div>
      <div className="version-history-body">
        <div className="version-list">
          {loading && <div className="version-list-empty">불러오는 중...</div>}
          {!loading && versions.length === 0 && (
            <div className="version-list-empty">저장된 버전이 없습니다</div>
          )}
          {/* 현재 버전 (저장되지 않은 최신 상태) */}
          {!loading && versions.length > 0 && (
            <div className="version-list-section">
              {versions.map((v) => (
                <button
                  key={v.id}
                  className={`version-item${selectedId === v.id ? ' active' : ''}`}
                  onClick={() => setSelectedId(v.id)}
                >
                  <span className="version-number">v{v.version_number}</span>
                  <span className="version-date">
                    {new Date(v.created_at).toLocaleString('ko-KR', {
                      month: 'short',
                      day: 'numeric',
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </span>
                  <span className="version-chars">{v.char_count}자</span>
                </button>
              ))}
            </div>
          )}
        </div>
        <div className="version-diff">
          {selectedVersion && diffResult.length > 0 ? (
            <>
              <div className="version-diff-header">
                v{selectedVersion.version_number} 변경 내역
              </div>
              <pre className="version-diff-content">
                {diffResult.map((part, i) => (
                  <span
                    key={i}
                    className={
                      part.added
                        ? 'diff-added'
                        : part.removed
                          ? 'diff-removed'
                          : 'diff-unchanged'
                    }
                  >
                    {part.value}
                  </span>
                ))}
              </pre>
            </>
          ) : selectedContent !== null ? (
            <>
              <div className="version-diff-header">
                v{selectedVersion?.version_number} 전체 내용
              </div>
              <pre className="version-diff-content">
                <span className="diff-unchanged">{selectedContent}</span>
              </pre>
            </>
          ) : (
            !loading && (
              <div className="version-diff-empty">버전을 선택하세요</div>
            )
          )}
        </div>
      </div>
    </div>
  );
}
