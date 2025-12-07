-- Seed test data for development

-- Insert test user (password: "password123")
-- BCrypt hash generated with: BCryptPasswordEncoder().encode("password123")
INSERT INTO users (id, email, username, password_hash, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'test@maestro.ai',
    'testuser',
    '$2a$10$rP7j0YLLdUZqN5Z.kHGxweGHC3vL.qVYVPi9LJf5/z5Z5ZQYKx5.u',
    NOW(),
    NOW()
);

-- Insert sample project
INSERT INTO projects (id, user_id, title, artist, bpm, time_signature, key, created_at, updated_at)
VALUES (
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000001',
    'Demo Song',
    'Test Artist',
    120,
    '4/4',
    'A',
    NOW(),
    NOW()
);

-- Insert sample tracks
INSERT INTO tracks (id, project_id, name, instrument, track_number, volume, pan, created_at, updated_at)
VALUES
(
    '00000000-0000-0000-0000-000000000003',
    '00000000-0000-0000-0000-000000000002',
    'Lead Guitar',
    'Electric Guitar',
    1,
    0.75,
    0.5,
    NOW(),
    NOW()
),
(
    '00000000-0000-0000-0000-000000000004',
    '00000000-0000-0000-0000-000000000002',
    'Rhythm Guitar',
    'Electric Guitar',
    2,
    0.70,
    0.3,
    NOW(),
    NOW()
),
(
    '00000000-0000-0000-0000-000000000005',
    '00000000-0000-0000-0000-000000000002',
    'Bass',
    'Bass Guitar',
    3,
    0.80,
    0.5,
    NOW(),
    NOW()
),
(
    '00000000-0000-0000-0000-000000000006',
    '00000000-0000-0000-0000-000000000002',
    'Drums',
    'Drums',
    4,
    0.85,
    0.5,
    NOW(),
    NOW()
);
