import { useState, type FormEvent } from 'react';
import { useNavigate, Link } from 'react-router-dom';
import { useAuthStore } from '@/stores/authStore';

const styles = {
  container: {
    display: 'flex',
    minHeight: '100vh',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#fafafa',
  } as const,
  card: {
    width: '100%',
    maxWidth: '400px',
    padding: '2rem',
    backgroundColor: '#fff',
    borderRadius: '8px',
    boxShadow: '0 1px 3px rgba(0,0,0,0.1)',
  } as const,
  title: {
    fontSize: '1.5rem',
    fontWeight: 600,
    textAlign: 'center' as const,
    marginBottom: '0.25rem',
  } as const,
  subtitle: {
    fontSize: '0.875rem',
    color: '#666',
    textAlign: 'center' as const,
    marginBottom: '1.5rem',
  } as const,
  label: {
    display: 'block',
    fontSize: '0.875rem',
    fontWeight: 500,
    marginBottom: '0.25rem',
    color: '#333',
  } as const,
  input: {
    width: '100%',
    padding: '0.5rem 0.75rem',
    border: '1px solid #ddd',
    borderRadius: '6px',
    fontSize: '0.875rem',
    outline: 'none',
    boxSizing: 'border-box' as const,
    marginBottom: '1rem',
  } as const,
  button: {
    width: '100%',
    padding: '0.625rem',
    backgroundColor: '#111',
    color: '#fff',
    border: 'none',
    borderRadius: '6px',
    fontSize: '0.875rem',
    fontWeight: 500,
    cursor: 'pointer',
  } as const,
  buttonDisabled: {
    opacity: 0.6,
    cursor: 'not-allowed',
  } as const,
  error: {
    color: '#dc2626',
    fontSize: '0.8125rem',
    marginBottom: '1rem',
    padding: '0.5rem',
    backgroundColor: '#fef2f2',
    borderRadius: '4px',
  } as const,
  link: {
    display: 'block',
    textAlign: 'center' as const,
    marginTop: '1rem',
    fontSize: '0.8125rem',
    color: '#666',
  } as const,
};

export function RegisterPage() {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [localError, setLocalError] = useState<string | null>(null);
  const { register, loading, error } = useAuthStore();
  const navigate = useNavigate();

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setLocalError(null);

    if (password !== confirmPassword) {
      setLocalError('비밀번호가 일치하지 않습니다');
      return;
    }
    if (password.length < 8) {
      setLocalError('비밀번호는 8자 이상이어야 합니다');
      return;
    }

    try {
      await register(username, password);
      navigate('/');
    } catch {
      // error is set in store
    }
  };

  const displayError = localError || error;

  return (
    <div style={styles.container}>
      <div style={styles.card}>
        <h1 style={styles.title}>Tecindo</h1>
        <p style={styles.subtitle}>새 계정을 만드세요</p>
        <form onSubmit={handleSubmit}>
          {displayError && <div style={styles.error}>{displayError}</div>}
          <label style={styles.label} htmlFor="username">사용자명</label>
          <input
            id="username"
            style={styles.input}
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            autoComplete="username"
          />
          <label style={styles.label} htmlFor="password">비밀번호</label>
          <input
            id="password"
            style={styles.input}
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            autoComplete="new-password"
          />
          <label style={styles.label} htmlFor="confirmPassword">비밀번호 확인</label>
          <input
            id="confirmPassword"
            style={styles.input}
            type="password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
            required
            autoComplete="new-password"
          />
          <button
            type="submit"
            style={{ ...styles.button, ...(loading ? styles.buttonDisabled : {}) }}
            disabled={loading}
          >
            {loading ? '가입 중...' : '회원가입'}
          </button>
        </form>
        <div style={styles.link}>
          이미 계정이 있으신가요? <Link to="/login">로그인</Link>
        </div>
      </div>
    </div>
  );
}
