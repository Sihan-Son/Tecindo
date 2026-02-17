import { useEffect, useRef, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import { Markdown } from 'tiptap-markdown';
import { useDocumentStore } from '@/stores/documentStore';
import { fetchDocumentContent } from '@/api/client';
import { TagSelector } from '@/components/Tags/TagSelector';

export default function Editor() {
  const { currentDocument, saveContent, updateDocument } = useDocumentStore();
  const [title, setTitle] = useState('');
  const [loadError, setLoadError] = useState<string | null>(null);
  const [exportingPdf, setExportingPdf] = useState(false);
  const [saveStatus, setSaveStatus] = useState<'idle' | 'saving' | 'saved'>('idle');
  const saveTimeoutRef = useRef<number | null>(null);
  const savedTimerRef = useRef<number | null>(null);
  const skipUpdateRef = useRef(false);
  const docIdRef = useRef<string | null>(null);

  useEffect(() => {
    docIdRef.current = currentDocument?.id ?? null;
  }, [currentDocument?.id]);

  const editor = useEditor({
    extensions: [
      StarterKit,
      Markdown,
    ],
    content: '',
    editorProps: {
      attributes: {
        class: 'editor-content',
      },
    },
    onUpdate: ({ editor }) => {
      const id = docIdRef.current;
      if (!id || skipUpdateRef.current) return;

      const markdown = editor.storage.markdown.getMarkdown();

      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }

      saveTimeoutRef.current = window.setTimeout(async () => {
        setSaveStatus('saving');
        await saveContent(id, markdown);
        setSaveStatus('saved');
        if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
        savedTimerRef.current = window.setTimeout(() => setSaveStatus('idle'), 2000);
      }, 1000);
    },
  });

  useEffect(() => {
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = null;
    }

    setLoadError(null);

    if (!currentDocument) {
      setTitle('');
      if (editor) {
        skipUpdateRef.current = true;
        editor.commands.setContent('');
        skipUpdateRef.current = false;
      }
      return;
    }

    const docId = currentDocument.id;
    setTitle(currentDocument.title);

    if (editor) {
      skipUpdateRef.current = true;
      editor.commands.setContent('');
      skipUpdateRef.current = false;
    }

    let cancelled = false;

    fetchDocumentContent(docId).then((content) => {
      if (cancelled || !editor) return;
      skipUpdateRef.current = true;
      editor.commands.setContent(content);
      skipUpdateRef.current = false;
    }).catch((err) => {
      if (!cancelled) {
        console.error('Failed to load document content:', err);
        setLoadError((err as Error).message || 'Failed to load content');
      }
    });

    return () => { cancelled = true; };
  }, [currentDocument?.id, editor]);

  const handleSave = async () => {
    if (!editor || !docIdRef.current) return;
    if (saveTimeoutRef.current) {
      clearTimeout(saveTimeoutRef.current);
      saveTimeoutRef.current = null;
    }
    const markdown = editor.storage.markdown.getMarkdown();
    setSaveStatus('saving');
    await saveContent(docIdRef.current, markdown);
    setSaveStatus('saved');
    if (savedTimerRef.current) clearTimeout(savedTimerRef.current);
    savedTimerRef.current = window.setTimeout(() => setSaveStatus('idle'), 2000);
  };

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
  };

  const handleTitleBlur = () => {
    if (currentDocument && title !== currentDocument.title) {
      updateDocument(currentDocument.id, { title });
    }
  };

  const text = editor?.getText() || '';
  const wordCount = text.split(/\s+/).filter(Boolean).length;
  const chars = [...text.replace(/\s/g, '')];
  const totalChars = chars.length;
  const koreanChars = chars.filter(c => /[\u3131-\u318E\uAC00-\uD7A3]/.test(c)).length;
  const englishChars = chars.filter(c => /[a-zA-Z]/.test(c)).length;
  const otherChars = totalChars - koreanChars - englishChars;

  const handleExportMarkdown = () => {
    if (!editor || !currentDocument) return;
    const markdown = editor.storage.markdown.getMarkdown();
    const slug = (title || 'untitled').replace(/[^a-zA-Z0-9가-힣ㄱ-ㅎㅏ-ㅣ _-]/g, '').replace(/\s+/g, '-');
    const blob = new Blob([markdown], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${slug}.md`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleExportPdf = async () => {
    if (!currentDocument || exportingPdf) return;
    setExportingPdf(true);
    try {
      const response = await fetch(`/api/v1/documents/${currentDocument.id}/export/pdf`);
      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: { message: 'PDF 변환 실패' } }));
        throw new Error(error.error?.message || `HTTP ${response.status}`);
      }
      const blob = await response.blob();
      const url = URL.createObjectURL(blob);
      const slug = (title || 'untitled').replace(/[^a-zA-Z0-9가-힣ㄱ-ㅎㅏ-ㅣ _-]/g, '').replace(/\s+/g, '-');
      const a = document.createElement('a');
      a.href = url;
      a.download = `${slug}.pdf`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (error) {
      alert((error as Error).message || 'PDF 변환에 실패했습니다');
    } finally {
      setExportingPdf(false);
    }
  };

  if (!currentDocument) {
    return (
      <div className="editor-container">
        <div className="editor-empty">
          <div className="onboarding">
            <h2 className="onboarding-title">Tecindo</h2>
            <p className="onboarding-subtitle">개인용 글쓰기 앱</p>
            <div className="onboarding-shortcuts">
              <div className="shortcut-item">
                <span className="shortcut-icon">+</span>
                <div>
                  <strong>새 문서</strong>
                  <p>사이드바 상단의 + 버튼으로 새 글을 시작하세요</p>
                </div>
              </div>
              <div className="shortcut-item">
                <span className="shortcut-icon">
                  <svg width="16" height="16" viewBox="0 0 20 20" fill="none"><path d="M2 5C2 3.89543 2.89543 3 4 3H7.17157C7.70201 3 8.21071 3.21071 8.58579 3.58579L9.41421 4.41421C9.78929 4.78929 10.298 5 10.8284 5H16C17.1046 5 18 5.89543 18 7V15C18 16.1046 17.1046 17 16 17H4C2.89543 17 2 16.1046 2 15V5Z" stroke="currentColor" strokeWidth="1.5"/><path d="M10 9V13M8 11H12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>
                </span>
                <div>
                  <strong>폴더 정리</strong>
                  <p>폴더를 만들고 문서를 드래그하여 분류하세요</p>
                </div>
              </div>
              <div className="shortcut-item">
                <span className="shortcut-icon">
                  <svg width="16" height="16" viewBox="0 0 16 16" fill="none"><path d="M8 2V10M8 10L5 7M8 10L11 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/><path d="M3 13H13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"/></svg>
                </span>
                <div>
                  <strong>.md 내보내기</strong>
                  <p>에디터 하단에서 마크다운 파일로 내보낼 수 있어요</p>
                </div>
              </div>
              <div className="shortcut-item">
                <span className="shortcut-icon">A</span>
                <div>
                  <strong>자동 저장</strong>
                  <p>작성한 내용은 1초 후 자동으로 저장됩니다</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="editor-container">
      <div className="editor-header">
        <input
          type="text"
          className="editor-title"
          value={title}
          onChange={handleTitleChange}
          onBlur={handleTitleBlur}
          placeholder="Untitled"
        />
        <div className="editor-tags">
          <TagSelector documentId={currentDocument.id} />
        </div>
      </div>
      {loadError && (
        <div className="editor-error">
          <p>Failed to load content: {loadError}</p>
          <button onClick={() => {
            if (currentDocument) {
              setLoadError(null);
              fetchDocumentContent(currentDocument.id).then((content) => {
                if (editor) {
                  skipUpdateRef.current = true;
                  editor.commands.setContent(content);
                  skipUpdateRef.current = false;
                }
              }).catch((err) => {
                setLoadError((err as Error).message || 'Failed to load content');
              });
            }
          }}>Retry</button>
        </div>
      )}
      <EditorContent editor={editor} />
      <div className="editor-footer">
        <div className="char-stats">
          <span className="char-stat-detail">{new Date(currentDocument.created_at).toLocaleDateString('ko-KR', { year: 'numeric', month: 'long', day: 'numeric' })} 작성</span>
          <span className="char-stat-detail">·</span>
          <span className="char-stat-total">{totalChars}자</span>
          <span className="char-stat-detail">한 {koreanChars} · 영 {englishChars}{otherChars > 0 ? ` · 기타 ${otherChars}` : ''} · {wordCount}단어</span>
          {saveStatus !== 'idle' && (
            <span className="save-status">{saveStatus === 'saving' ? '저장 중...' : '저장됨'}</span>
          )}
        </div>
        <div className="export-buttons">
          <button className="btn-save" onClick={handleSave} title="저장 (Ctrl+S)">저장</button>
          <button className="btn-export" onClick={handleExportMarkdown} title="Export as Markdown">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 2V10M8 10L5 7M8 10L11 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              <path d="M3 13H13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
            <span>.md</span>
          </button>
          <button className="btn-export" onClick={handleExportPdf} title="Export as PDF" disabled={exportingPdf}>
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
              <path d="M8 2V10M8 10L5 7M8 10L11 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
              <path d="M3 13H13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
            <span>{exportingPdf ? '...' : '.pdf'}</span>
          </button>
        </div>
      </div>
    </div>
  );
}
