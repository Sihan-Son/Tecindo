import { useEffect } from 'react';
import { useParams } from 'react-router-dom';
import Sidebar from '@/components/Sidebar/Sidebar';
import Editor from '@/components/Editor/Editor';
import { useUIStore } from '@/stores/uiStore';
import { useDocuments } from '@/hooks/useDocuments';

export default function Layout() {
  const { id } = useParams<{ id: string }>();
  const { sidebarOpen, toggleSidebar, theme, toggleTheme } = useUIStore();
  const { loadDocument } = useDocuments();

  useEffect(() => {
    if (id) {
      loadDocument(id);
    }
  }, [id]);

  return (
    <div className="layout">
      <div className="layout-toolbar">
        <button className="hamburger" onClick={toggleSidebar} aria-label="Toggle sidebar">
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
            <path d="M3 12H21M3 6H21M3 18H21" stroke="currentColor" strokeWidth="2" strokeLinecap="round" />
          </svg>
        </button>

        <button className="btn-theme-toggle" onClick={toggleTheme} aria-label="Toggle theme">
          {theme === 'light' ? (
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M10 2V3M10 17V18M18 10H17M3 10H2M15.66 15.66L14.95 14.95M5.05 5.05L4.34 4.34M15.66 4.34L14.95 5.05M5.05 14.95L4.34 15.66" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
              <circle cx="10" cy="10" r="4" stroke="currentColor" strokeWidth="1.5" />
            </svg>
          ) : (
            <svg width="20" height="20" viewBox="0 0 20 20" fill="none">
              <path d="M17.293 13.293A8 8 0 016.707 2.707a8.001 8.001 0 1010.586 10.586z" stroke="currentColor" strokeWidth="1.5" strokeLinejoin="round" />
            </svg>
          )}
        </button>
      </div>

      <div className={`layout-content ${sidebarOpen ? 'sidebar-open' : 'sidebar-closed'}`}>
        {sidebarOpen && <Sidebar />}
        <Editor />
      </div>
    </div>
  );
}
