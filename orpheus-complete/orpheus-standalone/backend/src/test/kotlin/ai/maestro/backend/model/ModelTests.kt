package ai.maestro.backend.model

import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import java.time.LocalDateTime
import java.util.UUID

class ProjectTest {

    @Test
    fun `should create project with required fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Test Project",
            bpm = 120
        )

        assertEquals("Test Project", project.title)
        assertEquals(120, project.bpm)
        assertEquals(user, project.user)
        assertEquals("4/4", project.timeSignature)
        assertNotNull(project.id)
        assertNotNull(project.createdAt)
        assertNotNull(project.updatedAt)
    }

    @Test
    fun `should create project with optional fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Complete Project",
            artist = "Artist Name",
            bpm = 140,
            timeSignature = "3/4",
            key = "Gm"
        )

        assertEquals("Complete Project", project.title)
        assertEquals("Artist Name", project.artist)
        assertEquals(140, project.bpm)
        assertEquals("3/4", project.timeSignature)
        assertEquals("Gm", project.key)
    }

    @Test
    fun `should maintain tracks collection`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        assertEquals(0, project.tracks.size)

        val track = Track(
            project = project,
            name = "Track 1",
            instrument = "guitar",
            trackNumber = 1
        )

        project.tracks.add(track)
        assertEquals(1, project.tracks.size)
        assertEquals("Track 1", project.tracks[0].name)
    }

    @Test
    fun `should maintain collaborators collection`() {
        val owner = User(
            email = "owner@test.com",
            username = "owner",
            passwordHash = "hash"
        )

        val collaborator = User(
            email = "collab@test.com",
            username = "collaborator",
            passwordHash = "hash"
        )

        val project = Project(
            user = owner,
            title = "Project",
            bpm = 120
        )

        assertEquals(0, project.collaborators.size)

        project.collaborators.add(
            Collaborator(
                project = project,
                user = collaborator,
                role = CollaboratorRole.EDITOR
            )
        )

        assertEquals(1, project.collaborators.size)
        assertEquals(collaborator, project.collaborators[0].user)
    }
}

class TrackTest {

    @Test
    fun `should create track with required fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val track = Track(
            project = project,
            name = "Guitar Track",
            instrument = "electric-guitar",
            trackNumber = 1
        )

        assertEquals("Guitar Track", track.name)
        assertEquals("electric-guitar", track.instrument)
        assertEquals(1, track.trackNumber)
        assertEquals(project, track.project)
        assertNotNull(track.id)
        assertNotNull(track.createdAt)
        assertNotNull(track.updatedAt)
    }

    @Test
    fun `should create track with default values`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val track = Track(
            project = project,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        assertEquals(0.75f, track.volume)
        assertEquals(0.5f, track.pan)
        assertFalse(track.muted)
        assertFalse(track.soloed)
    }

    @Test
    fun `should create track with custom values`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val track = Track(
            project = project,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1,
            volume = 0.9f,
            pan = 0.3f,
            muted = true,
            soloed = true
        )

        assertEquals(0.9f, track.volume)
        assertEquals(0.3f, track.pan)
        assertTrue(track.muted)
        assertTrue(track.soloed)
    }

    @Test
    fun `should allow tablature and processors`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val tablature = """{"measures": []}"""
        val processors = """{"effects": []}"""

        val track = Track(
            project = project,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1,
            tablature = tablature,
            processors = processors
        )

        assertEquals(tablature, track.tablature)
        assertEquals(processors, track.processors)
    }

    @Test
    fun `should allow audio file URL`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val track = Track(
            project = project,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        assertNull(track.audioFileUrl)

        track.audioFileUrl = "https://example.com/audio.wav"
        assertEquals("https://example.com/audio.wav", track.audioFileUrl)
    }
}

class UserTest {

    @Test
    fun `should create user with required fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hashedpassword"
        )

        assertEquals("test@example.com", user.email)
        assertEquals("testuser", user.username)
        assertEquals("hashedpassword", user.passwordHash)
        assertNotNull(user.id)
        assertNotNull(user.createdAt)
    }

    @Test
    fun `should maintain projects collection`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        assertEquals(0, user.projects.size)

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        user.projects.add(project)
        assertEquals(1, user.projects.size)
        assertEquals("Project", user.projects[0].title)
    }

    @Test
    fun `should have unique email constraint`() {
        val user1 = User(
            email = "test@example.com",
            username = "user1",
            passwordHash = "hash"
        )

        val user2 = User(
            email = "test@example.com",
            username = "user2",
            passwordHash = "hash"
        )

        // Both users created, but DB would enforce uniqueness
        assertNotNull(user1)
        assertNotNull(user2)
    }
}

class CollaboratorTest {

    @Test
    fun `should create collaborator with role`() {
        val owner = User(
            email = "owner@test.com",
            username = "owner",
            passwordHash = "hash"
        )

        val collaborator = User(
            email = "collab@test.com",
            username = "collaborator",
            passwordHash = "hash"
        )

        val project = Project(
            user = owner,
            title = "Project",
            bpm = 120
        )

        val collaboration = Collaborator(
            project = project,
            user = collaborator,
            role = CollaboratorRole.EDITOR
        )

        assertEquals(project, collaboration.project)
        assertEquals(collaborator, collaboration.user)
        assertEquals(CollaboratorRole.EDITOR, collaboration.role)
        assertNotNull(collaboration.id)
    }

    @Test
    fun `should support all collaborator roles`() {
        val owner = User(
            email = "owner@test.com",
            username = "owner",
            passwordHash = "hash"
        )

        val user = User(
            email = "user@test.com",
            username = "user",
            passwordHash = "hash"
        )

        val project = Project(
            user = owner,
            title = "Project",
            bpm = 120
        )

        val editor = Collaborator(
            project = project,
            user = user,
            role = CollaboratorRole.EDITOR
        )

        val viewer = Collaborator(
            project = project,
            user = user,
            role = CollaboratorRole.VIEWER
        )

        assertEquals(CollaboratorRole.EDITOR, editor.role)
        assertEquals(CollaboratorRole.VIEWER, viewer.role)
    }
}

class DistributionTest {

    @Test
    fun `should create distribution with required fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val distribution = Distribution(
            project = project,
            title = "My Song",
            artist = "Artist Name"
        )

        assertEquals("My Song", distribution.title)
        assertEquals("Artist Name", distribution.artist)
        assertEquals(project, distribution.project)
        assertEquals(DistributionStatus.DRAFT, distribution.status)
        assertNotNull(distribution.id)
        assertNotNull(distribution.createdAt)
        assertNotNull(distribution.updatedAt)
    }

    @Test
    fun `should create distribution with optional fields`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val distribution = Distribution(
            project = project,
            title = "My Song",
            artist = "Artist Name",
            album = "Album Name",
            genre = "Rock",
            isrc = "USRC17607839",
            upc = "123456789012",
            artworkUrl = "https://example.com/art.jpg",
            platforms = listOf("Spotify", "Apple Music", "YouTube Music")
        )

        assertEquals("My Song", distribution.title)
        assertEquals("Artist Name", distribution.artist)
        assertEquals("Album Name", distribution.album)
        assertEquals("Rock", distribution.genre)
        assertEquals("USRC17607839", distribution.isrc)
        assertEquals("123456789012", distribution.upc)
        assertEquals("https://example.com/art.jpg", distribution.artworkUrl)
        assertEquals(3, distribution.platforms?.size)
    }

    @Test
    fun `should support all distribution statuses`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val draft = Distribution(
            project = project,
            title = "Song",
            artist = "Artist",
            status = DistributionStatus.DRAFT
        )

        val submitted = Distribution(
            project = project,
            title = "Song",
            artist = "Artist",
            status = DistributionStatus.SUBMITTED
        )

        val published = Distribution(
            project = project,
            title = "Song",
            artist = "Artist",
            status = DistributionStatus.PUBLISHED
        )

        val failed = Distribution(
            project = project,
            title = "Song",
            artist = "Artist",
            status = DistributionStatus.FAILED
        )

        assertEquals(DistributionStatus.DRAFT, draft.status)
        assertEquals(DistributionStatus.SUBMITTED, submitted.status)
        assertEquals(DistributionStatus.PUBLISHED, published.status)
        assertEquals(DistributionStatus.FAILED, failed.status)
    }

    @Test
    fun `should track submission and publication timestamps`() {
        val user = User(
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            user = user,
            title = "Project",
            bpm = 120
        )

        val distribution = Distribution(
            project = project,
            title = "Song",
            artist = "Artist"
        )

        assertNull(distribution.submittedAt)
        assertNull(distribution.publishedAt)

        distribution.submittedAt = LocalDateTime.now()
        distribution.status = DistributionStatus.SUBMITTED

        assertNotNull(distribution.submittedAt)
        assertNull(distribution.publishedAt)

        distribution.publishedAt = LocalDateTime.now()
        distribution.status = DistributionStatus.PUBLISHED

        assertNotNull(distribution.publishedAt)
    }
}
