import { useEffect, useRef, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import { Markdown } from 'tiptap-markdown';
import { useDocumentStore } from '@/stores/documentStore';
import { fetchDocumentContent } from '@/api/client';

export default function Editor() {
  const { currentDocument, saveContent, updateDocument } = useDocumentStore();
  const [title, setTitle] = useState('');
  const [loadError, setLoadError] = useState<string | null>(null);
  const saveTimeoutRef = useRef<number | null>(null);
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

      saveTimeoutRef.current = window.setTimeout(() => {
        saveContent(id, markdown);
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

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
  };

  const handleTitleBlur = () => {
    if (currentDocument && title !== currentDocument.title) {
      updateDocument(currentDocument.id, { title });
    }
  };

  const wordCount = editor?.getText().split(/\s+/).filter(Boolean).length || 0;

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

  if (!currentDocument) {
    return (
      <div className="editor-container">
        <div className="editor-empty">
          <p>Select a document or create a new one to start writing</p>
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
        <span className="word-count">{wordCount} words</span>
        <button className="btn-export" onClick={handleExportMarkdown} title="Export as Markdown">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M8 2V10M8 10L5 7M8 10L11 7" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M3 13H13" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
          </svg>
          <span>.md</span>
        </button>
      </div>
    </div>
  );
}
