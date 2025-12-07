import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import './NewProject.css';

interface ProjectFormData {
  title: string;
  artist: string;
  bpm: number;
  timeSignature: string;
  key: string;
}

export const NewProject: React.FC = () => {
  const navigate = useNavigate();
  const [formData, setFormData] = useState<ProjectFormData>({
    title: '',
    artist: '',
    bpm: 120,
    timeSignature: '4/4',
    key: 'C',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      const response = await fetch('/api/v1/projects', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(formData),
      });

      if (!response.ok) throw new Error('Failed to create project');

      const project = await response.json();
      navigate(`/project/${project.id}`);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: name === 'bpm' ? parseInt(value) : value,
    }));
  };

  return (
    <div className="new-project-container">
      <div className="new-project-card">
        <h1>Create New Project</h1>

        {error && (
          <div className="error-message" data-testid="error-message">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="new-project-form">
          <div className="form-group">
            <label htmlFor="title">Project Title *</label>
            <input
              type="text"
              id="title"
              name="title"
              data-testid="project-title-input"
              value={formData.title}
              onChange={handleChange}
              required
              autoFocus
              placeholder="My Awesome Song"
            />
          </div>

          <div className="form-group">
            <label htmlFor="artist">Artist Name</label>
            <input
              type="text"
              id="artist"
              name="artist"
              data-testid="project-artist-input"
              value={formData.artist}
              onChange={handleChange}
              placeholder="Artist Name"
            />
          </div>

          <div className="form-row">
            <div className="form-group">
              <label htmlFor="bpm">BPM *</label>
              <input
                type="number"
                id="bpm"
                name="bpm"
                data-testid="project-bpm-input"
                value={formData.bpm}
                onChange={handleChange}
                min="20"
                max="300"
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="timeSignature">Time Signature *</label>
              <select
                id="timeSignature"
                name="timeSignature"
                data-testid="project-time-signature-input"
                value={formData.timeSignature}
                onChange={handleChange}
                required
              >
                <option value="4/4">4/4</option>
                <option value="3/4">3/4</option>
                <option value="6/8">6/8</option>
                <option value="5/4">5/4</option>
                <option value="7/8">7/8</option>
              </select>
            </div>

            <div className="form-group">
              <label htmlFor="key">Key</label>
              <select
                id="key"
                name="key"
                data-testid="project-key-input"
                value={formData.key}
                onChange={handleChange}
              >
                <option value="">None</option>
                <option value="C">C Major</option>
                <option value="Cm">C Minor</option>
                <option value="G">G Major</option>
                <option value="Gm">G Minor</option>
                <option value="D">D Major</option>
                <option value="Dm">D Minor</option>
                <option value="A">A Major</option>
                <option value="Am">A Minor</option>
                <option value="E">E Major</option>
                <option value="Em">E Minor</option>
                <option value="B">B Major</option>
                <option value="Bm">B Minor</option>
                <option value="F#">F# Major</option>
                <option value="F#m">F# Minor</option>
                <option value="Db">Db Major</option>
                <option value="C#m">C# Minor</option>
                <option value="Ab">Ab Major</option>
                <option value="Abm">Ab Minor</option>
                <option value="Eb">Eb Major</option>
                <option value="Ebm">Eb Minor</option>
                <option value="Bb">Bb Major</option>
                <option value="Bbm">Bb Minor</option>
                <option value="F">F Major</option>
                <option value="Fm">F Minor</option>
              </select>
            </div>
          </div>

          <div className="form-actions">
            <button
              type="button"
              className="btn-secondary"
              onClick={() => navigate('/projects')}
              disabled={loading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn-primary"
              data-testid="create-project-submit"
              disabled={loading}
            >
              {loading ? 'Creating...' : 'Create Project'}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
};
