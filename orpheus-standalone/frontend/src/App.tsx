import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { ProjectList } from './components/ProjectList';
import { NewProject } from './components/NewProject';
import { Workspace } from './components/Workspace';
import './App.css';

const App: React.FC = () => {
  return (
    <Router>
      <div className="app" data-testid="app">
        <Routes>
          <Route path="/" element={<Navigate to="/projects" replace />} />
          <Route path="/projects" element={<ProjectList />} />
          <Route path="/projects/new" element={<NewProject />} />
          <Route path="/project/:projectId" element={<Workspace />} />
        </Routes>
      </div>
    </Router>
  );
};

export default App;
