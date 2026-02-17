import { useState, useEffect, useRef, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { useDocumentStore } from '@/stores/documentStore';
import type { Document, Folder } from '@/lib/types';

function buildFolderTree(folders: Folder[], parentId: string | null = null): Folder[] {
  return folders
    .filter((f) => f.parent_id === parentId)
    .sort((a, b) => a.sort_order - b.sort_order);
}

export default function Sidebar() {
  const {
    documents,
    currentDocument,
    folders,
    currentFolderId,
    createDocument,
    loadDocument,
    updateDocument,
    deleteDocument,
    loadFolders,
    createFolder,
    updateFolder,
    deleteFolder,
    setCurrentFolderId,
  } = useDocumentStore();
  const [searchQuery, setSearchQuery] = useState('');
  const [newFolderName, setNewFolderName] = useState('');
  const [showNewFolder, setShowNewFolder] = useState(false);
  const [editingFolderId, setEditingFolderId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState('');
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set());
  const [menuDocId, setMenuDocId] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const newFolderInputRef = useRef<HTMLInputElement>(null);
  const editInputRef = useRef<HTMLInputElement>(null);
  const navigate = useNavigate();

  const closeMenu = useCallback(() => setMenuDocId(null), []);

  useEffect(() => {
    if (!menuDocId) return;
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        closeMenu();
      }
    };
    document.addEventListener('mousedown', handleClick);
    return () => document.removeEventListener('mousedown', handleClick);
  }, [menuDocId, closeMenu]);

  useEffect(() => {
    loadFolders();
  }, []);

  useEffect(() => {
    if (showNewFolder && newFolderInputRef.current) {
      newFolderInputRef.current.focus();
    }
  }, [showNewFolder]);

  useEffect(() => {
    if (editingFolderId && editInputRef.current) {
      editInputRef.current.focus();
    }
  }, [editingFolderId]);

  const filteredDocuments = documents.filter((doc) => {
    const matchesSearch = doc.title.toLowerCase().includes(searchQuery.toLowerCase());
    const matchesFolder = currentFolderId === null || doc.folder_id === currentFolderId;
    return matchesSearch && matchesFolder;
  });

  const handleNewDocument = async () => {
    try {
      const doc = await createDocument(
        currentFolderId ? { folder_id: currentFolderId } : undefined
      );
      navigate(`/doc/${doc.id}`);
      await loadDocument(doc.id);
    } catch (error) {
      console.error('Failed to create document:', error);
    }
  };

  const handleDocumentClick = async (doc: Document) => {
    navigate(`/doc/${doc.id}`);
    await loadDocument(doc.id);
  };

  const handleMoveDocument = async (docId: string, folderId: string | null) => {
    await updateDocument(docId, { folder_id: folderId });
    closeMenu();
  };

  const handleDeleteDocument = async (docId: string) => {
    if (!confirm('이 문서를 삭제하시겠습니까?')) return;
    await deleteDocument(docId);
    closeMenu();
  };

  const renderMoveTree = (docId: string, parentId: string | null, depth: number): React.ReactNode => {
    const children = buildFolderTree(folders, parentId);
    if (children.length === 0) return null;
    return children.map((folder) => (
      <div key={folder.id}>
        <button
          className="document-dropdown-item"
          style={{ paddingLeft: `${12 + depth * 16}px` }}
          onClick={() => handleMoveDocument(docId, folder.id)}
        >
          <svg width="12" height="12" viewBox="0 0 14 14" fill="none" style={{ flexShrink: 0 }}>
            <path d="M1.5 3C1.5 2.44772 1.94772 2 2.5 2H5.29289C5.4255 2 5.55268 2.05268 5.64645 2.14645L6.85355 3.35355C6.94732 3.44732 7.0745 3.5 7.20711 3.5H11.5C12.0523 3.5 12.5 3.94772 12.5 4.5V11C12.5 11.5523 12.0523 12 11.5 12H2.5C1.94772 12 1.5 11.5523 1.5 11V3Z" stroke="currentColor" strokeWidth="1.2" />
          </svg>
          {folder.name}
        </button>
        {renderMoveTree(docId, folder.id, depth + 1)}
      </div>
    ));
  };

  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    try {
      await createFolder({
        name: newFolderName.trim(),
        parent_id: currentFolderId || undefined,
      });
      setNewFolderName('');
      setShowNewFolder(false);
    } catch (error) {
      console.error('Failed to create folder:', error);
    }
  };

  const handleRenameFolder = async (id: string) => {
    if (!editingName.trim()) {
      setEditingFolderId(null);
      return;
    }
    try {
      await updateFolder(id, { name: editingName.trim() });
      setEditingFolderId(null);
    } catch (error) {
      console.error('Failed to rename folder:', error);
    }
  };

  const handleDeleteFolder = async (id: string) => {
    if (!confirm('Delete this folder? Documents inside will be unlinked.')) return;
    try {
      await deleteFolder(id);
    } catch (error) {
      console.error('Failed to delete folder:', error);
    }
  };

  const toggleFolderExpand = (id: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  const handleFolderClick = (id: string | null) => {
    setCurrentFolderId(id);
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) return 'Today';
    if (days === 1) return 'Yesterday';
    if (days < 7) return `${days} days ago`;
    return date.toLocaleDateString();
  };

  const renderFolderTree = (parentId: string | null, depth: number = 0) => {
    const children = buildFolderTree(folders, parentId);
    if (children.length === 0) return null;

    return children.map((folder) => {
      const isExpanded = expandedFolders.has(folder.id);
      const hasChildren = folders.some((f) => f.parent_id === folder.id);
      const isActive = currentFolderId === folder.id;

      return (
        <div key={folder.id} className="folder-tree-item">
          <div
            className={`folder-row ${isActive ? 'active' : ''}`}
            style={{ paddingLeft: `${12 + depth * 16}px` }}
          >
            <button
              className="folder-expand"
              onClick={() => toggleFolderExpand(folder.id)}
              style={{ visibility: hasChildren ? 'visible' : 'hidden' }}
            >
              <svg width="12" height="12" viewBox="0 0 12 12" fill="none"
                style={{ transform: isExpanded ? 'rotate(90deg)' : 'none', transition: 'transform 0.15s' }}
              >
                <path d="M4 2L8 6L4 10" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              </svg>
            </button>

            {editingFolderId === folder.id ? (
              <input
                ref={editInputRef}
                className="folder-rename-input"
                value={editingName}
                onChange={(e) => setEditingName(e.target.value)}
                onBlur={() => handleRenameFolder(folder.id)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleRenameFolder(folder.id);
                  if (e.key === 'Escape') setEditingFolderId(null);
                }}
              />
            ) : (
              <button
                className="folder-name"
                onClick={() => handleFolderClick(folder.id)}
                onDoubleClick={() => {
                  setEditingFolderId(folder.id);
                  setEditingName(folder.name);
                }}
              >
                <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                  <path d="M1.5 3C1.5 2.44772 1.94772 2 2.5 2H5.29289C5.4255 2 5.55268 2.05268 5.64645 2.14645L6.85355 3.35355C6.94732 3.44732 7.0745 3.5 7.20711 3.5H11.5C12.0523 3.5 12.5 3.94772 12.5 4.5V11C12.5 11.5523 12.0523 12 11.5 12H2.5C1.94772 12 1.5 11.5523 1.5 11V3Z" stroke="currentColor" strokeWidth="1.2" />
                </svg>
                <span>{folder.name}</span>
              </button>
            )}

            <div className="folder-actions">
              <button
                className="folder-action-btn"
                onClick={() => {
                  setEditingFolderId(folder.id);
                  setEditingName(folder.name);
                }}
                title="Rename"
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                  <path d="M8.5 1.5L10.5 3.5L4 10H2V8L8.5 1.5Z" stroke="currentColor" strokeWidth="1.2" strokeLinejoin="round" />
                </svg>
              </button>
              <button
                className="folder-action-btn"
                onClick={() => handleDeleteFolder(folder.id)}
                title="Delete"
              >
                <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                  <path d="M3 3L9 9M9 3L3 9" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
                </svg>
              </button>
            </div>
          </div>
          {isExpanded && renderFolderTree(folder.id, depth + 1)}
        </div>
      );
    });
  };

  return (
    <div className="sidebar">
      <div className="sidebar-header">
        <h1 className="sidebar-title">Tecindo</h1>
        <div className="sidebar-header-actions">
          <button
            className="btn-icon"
            onClick={() => setShowNewFolder(true)}
            title="New Folder"
          >
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M2 5C2 3.89543 2.89543 3 4 3H7.17157C7.70201 3 8.21071 3.21071 8.58579 3.58579L9.41421 4.41421C9.78929 4.78929 10.298 5 10.8284 5H16C17.1046 5 18 5.89543 18 7V15C18 16.1046 17.1046 17 16 17H4C2.89543 17 2 16.1046 2 15V5Z" stroke="currentColor" strokeWidth="1.5" />
              <path d="M10 9V13M8 11H12" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
            </svg>
          </button>
          <button className="btn-new-document" onClick={handleNewDocument} title="New Document">
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M10 4V16M4 10H16" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
            </svg>
          </button>
        </div>
      </div>

      <div className="sidebar-search">
        <input
          type="text"
          placeholder="Search documents..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="search-input"
        />
      </div>

      <div className="sidebar-folders">
        <button
          className={`folder-row all-documents ${currentFolderId === null ? 'active' : ''}`}
          onClick={() => handleFolderClick(null)}
        >
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
            <path d="M2 3.5H12M2 7H12M2 10.5H12" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
          </svg>
          <span>All Documents</span>
        </button>

        {renderFolderTree(null)}

        {showNewFolder && (
          <div className="folder-row new-folder-row">
            <input
              ref={newFolderInputRef}
              className="folder-rename-input"
              value={newFolderName}
              placeholder="Folder name"
              onChange={(e) => setNewFolderName(e.target.value)}
              onBlur={() => {
                if (newFolderName.trim()) handleCreateFolder();
                else setShowNewFolder(false);
              }}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleCreateFolder();
                if (e.key === 'Escape') setShowNewFolder(false);
              }}
            />
          </div>
        )}
      </div>

      <div className="sidebar-documents">
        {filteredDocuments.length === 0 ? (
          <div className="sidebar-empty">
            <p>No documents found</p>
          </div>
        ) : (
          filteredDocuments.map((doc) => (
            <div key={doc.id} className="document-item-wrapper">
              <button
                className={`document-item ${currentDocument?.id === doc.id ? 'active' : ''}`}
                onClick={() => handleDocumentClick(doc)}
              >
                <div className="document-title">{doc.title || 'Untitled'}</div>
                {doc.excerpt && <div className="document-excerpt">{doc.excerpt}</div>}
                <div className="document-meta">
                  <span className="document-date">{formatDate(doc.updated_at)}</span>
                  {doc.word_count > 0 && (
                    <span className="document-words">{doc.word_count} words</span>
                  )}
                </div>
              </button>
              <div className="document-more-wrapper">
                <button
                  className="document-more-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    setMenuDocId(menuDocId === doc.id ? null : doc.id);
                  }}
                  title="More actions"
                >
                  <svg width="14" height="14" viewBox="0 0 14 14" fill="none">
                    <circle cx="7" cy="3" r="1.2" fill="currentColor" />
                    <circle cx="7" cy="7" r="1.2" fill="currentColor" />
                    <circle cx="7" cy="11" r="1.2" fill="currentColor" />
                  </svg>
                </button>
                {menuDocId === doc.id && (
                  <div ref={menuRef} className="document-dropdown">
                    {folders.length > 0 && (
                      <>
                        <div className="document-dropdown-label">Move to</div>
                        <button
                          className="document-dropdown-item"
                          onClick={() => handleMoveDocument(doc.id, null)}
                        >
                          Root (No folder)
                        </button>
                        {renderMoveTree(doc.id, null, 0)}
                        <div className="document-dropdown-divider" />
                      </>
                    )}
                    <button
                      className="document-dropdown-item document-dropdown-delete"
                      onClick={() => handleDeleteDocument(doc.id)}
                    >
                      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
                        <path d="M2 3H10M4.5 3V2.5C4.5 2.22386 4.72386 2 5 2H7C7.27614 2 7.5 2.22386 7.5 2.5V3M5 5.5V8.5M7 5.5V8.5M3 3L3.5 9.5C3.5 9.77614 3.72386 10 4 10H8C8.27614 10 8.5 9.77614 8.5 9.5L9 3" stroke="currentColor" strokeWidth="1" strokeLinecap="round" strokeLinejoin="round" />
                      </svg>
                      Delete
                    </button>
                  </div>
                )}
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}
