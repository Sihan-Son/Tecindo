import { useEffect, useRef, useState } from 'react';
import { useEditor, EditorContent } from '@tiptap/react';
import StarterKit from '@tiptap/starter-kit';
import { Markdown } from 'tiptap-markdown';
import { useDocumentStore } from '@/stores/documentStore';

export default function Editor() {
  const { currentDocument, currentContent, loadContent, saveContent, updateDocument } = useDocumentStore();
  const [title, setTitle] = useState('');
  const saveTimeoutRef = useRef<number | null>(null);

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
      if (!currentDocument) return;

      const markdown = editor.storage.markdown.getMarkdown();

      if (saveTimeoutRef.current) {
        clearTimeout(saveTimeoutRef.current);
      }

      saveTimeoutRef.current = window.setTimeout(() => {
        saveContent(currentDocument.id, markdown);
      }, 1000);
    },
  });

  useEffect(() => {
    if (currentDocument) {
      setTitle(currentDocument.title);
      loadContent(currentDocument.id);
    } else {
      setTitle('');
      editor?.commands.setContent('');
    }
  }, [currentDocument?.id]);

  useEffect(() => {
    if (editor && currentContent) {
      editor.commands.setContent(currentContent);
    }
  }, [currentContent, editor]);

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
  };

  const handleTitleBlur = () => {
    if (currentDocument && title !== currentDocument.title) {
      updateDocument(currentDocument.id, { title });
    }
  };

  const wordCount = editor?.getText().split(/\s+/).filter(Boolean).length || 0;

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
      <EditorContent editor={editor} />
      <div className="editor-footer">
        <span className="word-count">{wordCount} words</span>
      </div>
    </div>
  );
}
