import { useState, useEffect, type FormEvent } from 'react';
import { useTagStore } from '@/stores/tagStore';
import { TagBadge } from './TagBadge';

const TAG_COLORS = ['#ef4444', '#f97316', '#eab308', '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899', '#6b7280'];

const styles = {
  container: {
    padding: '1rem',
  },
  header: {
    fontSize: '1rem',
    fontWeight: 600,
    marginBottom: '1rem',
  },
  form: {
    display: 'flex',
    gap: '0.5rem',
    marginBottom: '1rem',
  },
  input: {
    flex: 1,
    padding: '0.375rem 0.5rem',
    border: '1px solid #ddd',
    borderRadius: '6px',
    fontSize: '0.8125rem',
    outline: 'none',
  },
  addBtn: {
    padding: '0.375rem 0.75rem',
    backgroundColor: '#111',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '0.8125rem',
    cursor: 'pointer',
  },
  colorPicker: {
    display: 'flex',
    gap: '0.25rem',
    marginBottom: '1rem',
  },
  colorDot: (color: string, selected: boolean) => ({
    width: '20px',
    height: '20px',
    borderRadius: '50%',
    backgroundColor: color,
    border: selected ? '2px solid #111' : '2px solid transparent',
    cursor: 'pointer',
    padding: 0,
  }),
  tagItem: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '0.375rem 0',
    borderBottom: '1px solid #f3f4f6',
  },
  deleteBtn: {
    background: 'none',
    border: 'none',
    color: '#999',
    cursor: 'pointer',
    fontSize: '0.8125rem',
  },
  list: {
    listStyle: 'none',
    margin: 0,
    padding: 0,
  },
};

export function TagManager() {
  const [name, setName] = useState('');
  const [color, setColor] = useState(TAG_COLORS[0]);
  const { tags, loading, loadTags, createTag, deleteTag } = useTagStore();

  useEffect(() => {
    loadTags();
  }, [loadTags]);

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    await createTag({ name: name.trim(), color });
    setName('');
  };

  return (
    <div style={styles.container}>
      <h3 style={styles.header}>태그 관리</h3>
      <form style={styles.form} onSubmit={handleSubmit}>
        <input
          style={styles.input}
          type="text"
          placeholder="새 태그 이름"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        <button style={styles.addBtn} type="submit" disabled={loading}>
          추가
        </button>
      </form>
      <div style={styles.colorPicker}>
        {TAG_COLORS.map((c) => (
          <button
            key={c}
            style={styles.colorDot(c, c === color)}
            onClick={() => setColor(c)}
            aria-label={`색상 ${c}`}
          />
        ))}
      </div>
      <ul style={styles.list}>
        {tags.map((tag) => (
          <li key={tag.id} style={styles.tagItem}>
            <TagBadge tag={tag} />
            <button
              style={styles.deleteBtn}
              onClick={() => deleteTag(tag.id)}
              aria-label={`${tag.name} 삭제`}
            >
              삭제
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}
