import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { formatDistanceToNow } from 'date-fns';
import './ProjectList.css';

interface Project {
  id: string;
  title: string;
  artist?: string;
  bpm: number;
  timeSignature: string;
  key?: string;
  trackCount: number;
  created: string;
  modified: string;
}

export const ProjectList: React.FC = () => {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();

  useEffect(() => {
    loadProjects();
  }, []);

  const loadProjects = async () => {
    try {
      const response = await fetch('/api/v1/projects');
      if (!response.ok) throw new Error('Failed to load projects');
      const data = await response.json();
      setProjects(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleOpenProject = (id: string) => {
    navigate(`/project/${id}`);
  };

  const handleDeleteProject = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm('Are you sure you want to delete this project?')) return;

    try {
      const response = await fetch(`/api/v1/projects/${id}`, {
        method: 'DELETE',
      });
      if (!response.ok) throw new Error('Failed to delete project');
      setProjects(projects.filter(p => p.id !== id));
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    }
  };

  if (loading) {
    return (
      <div className="project-list-loading" data-testid="project-list-loading">
        Loading projects...
      </div>
    );
  }

  if (error) {
    return (
      <div className="project-list-error" data-testid="project-list-error">
        Error: {error}
      </div>
    );
  }

  return (
    <div className="project-list-container">
      <div className="project-list-header">
        <h1>My Projects</h1>
        <button
          className="btn-primary"
          data-testid="new-project-button"
          onClick={() => navigate('/project/new')}
        >
          + New Project
        </button>
      </div>

      <div className="project-list" data-testid="project-list">
        {projects.length === 0 ? (
          <div className="project-list-empty">
            <p>No projects yet. Create your first project!</p>
          </div>
        ) : (
          projects.map(project => (
            <div
              key={project.id}
              className="project-card"
              data-testid="project-card"
              data-id={project.id}
              onClick={() => handleOpenProject(project.id)}
            >
              <div className="project-card-header">
                <h3 data-testid="project-card-title">{project.title}</h3>
                <button
                  className="btn-icon"
                  onClick={(e) => handleDeleteProject(project.id, e)}
                  aria-label="Delete project"
                >
                  🗑️
                </button>
              </div>

              <div className="project-card-meta">
                {project.artist && (
                  <span className="project-artist">{project.artist}</span>
                )}
                <span className="project-info">
                  {project.bpm} BPM • {project.timeSignature}
                  {project.key && ` • ${project.key}`}
                </span>
                <span className="project-tracks">
                  {project.trackCount} {project.trackCount === 1 ? 'track' : 'tracks'}
                </span>
              </div>

              <div className="project-card-footer">
                <span className="project-date" data-testid="project-card-date">
                  Modified {formatDistanceToNow(new Date(project.modified), { addSuffix: true })}
                </span>
              </div>
            </div>
          ))
        )}
      </div>
    </div>
  );
};
