package ai.maestro.backend.dto

import ai.maestro.backend.model.Project
import ai.maestro.backend.model.Track
import ai.maestro.backend.model.User
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.Test
import java.time.LocalDateTime
import java.util.UUID

class ProjectDtosTest {

    @Test
    fun `CreateProjectRequest should have default values`() {
        val request = CreateProjectRequest(title = "Test")

        assertEquals("Test", request.title)
        assertEquals(120, request.bpm)
        assertEquals("4/4", request.timeSignature)
        assertNull(request.artist)
        assertNull(request.key)
        assertNull(request.metadata)
    }

    @Test
    fun `CreateProjectRequest should accept all parameters`() {
        val metadata = mapOf("genre" to "Rock")
        val request = CreateProjectRequest(
            title = "My Song",
            artist = "Artist Name",
            bpm = 140,
            timeSignature = "3/4",
            key = "Gm",
            metadata = metadata
        )

        assertEquals("My Song", request.title)
        assertEquals("Artist Name", request.artist)
        assertEquals(140, request.bpm)
        assertEquals("3/4", request.timeSignature)
        assertEquals("Gm", request.key)
        assertEquals(metadata, request.metadata)
    }

    @Test
    fun `UpdateProjectRequest should allow partial updates`() {
        val request = UpdateProjectRequest(
            title = "Updated Title",
            bpm = 150
        )

        assertEquals("Updated Title", request.title)
        assertEquals(150, request.bpm)
        assertNull(request.artist)
        assertNull(request.timeSignature)
    }

    @Test
    fun `ProjectResponse from should map project correctly`() {
        val user = User(
            id = UUID.randomUUID(),
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            id = UUID.randomUUID(),
            user = user,
            title = "Test Project",
            artist = "Test Artist",
            bpm = 120,
            timeSignature = "4/4",
            key = "C"
        )

        val response = ProjectResponse.from(project)

        assertEquals(project.id, response.id)
        assertEquals(user.id, response.userId)
        assertEquals("Test Project", response.title)
        assertEquals("Test Artist", response.artist)
        assertEquals(120, response.bpm)
        assertEquals("4/4", response.timeSignature)
        assertEquals("C", response.key)
        assertEquals(0, response.trackCount)
    }

    @Test
    fun `ProjectResponse should include track count`() {
        val user = User(
            id = UUID.randomUUID(),
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            id = UUID.randomUUID(),
            user = user,
            title = "Test",
            bpm = 120
        )

        // Add tracks
        project.tracks.add(
            Track(
                id = UUID.randomUUID(),
                project = project,
                name = "Track 1",
                instrument = "Guitar",
                trackNumber = 1
            )
        )

        val response = ProjectResponse.from(project)
        assertEquals(1, response.trackCount)
    }

    @Test
    fun `ProjectDetailResponse should include tracks`() {
        val user = User(
            id = UUID.randomUUID(),
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            id = UUID.randomUUID(),
            user = user,
            title = "Test",
            bpm = 120
        )

        project.tracks.add(
            Track(
                id = UUID.randomUUID(),
                project = project,
                name = "Track 1",
                instrument = "Guitar",
                trackNumber = 1
            )
        )

        val response = ProjectDetailResponse.from(project)

        assertEquals(project.id, response.id)
        assertEquals(1, response.tracks.size)
        assertEquals("Track 1", response.tracks[0].name)
    }

    @Test
    fun `CreateTrackRequest should have default values`() {
        val request = CreateTrackRequest(
            name = "Guitar",
            instrument = "electric-guitar",
            trackNumber = 1
        )

        assertEquals("Guitar", request.name)
        assertEquals("electric-guitar", request.instrument)
        assertEquals(1, request.trackNumber)
        assertEquals(0.75f, request.volume)
        assertEquals(0.5f, request.pan)
        assertNull(request.tablature)
        assertNull(request.processors)
    }

    @Test
    fun `TrackResponse from should map track correctly`() {
        val user = User(
            id = UUID.randomUUID(),
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hash"
        )

        val project = Project(
            id = UUID.randomUUID(),
            user = user,
            title = "Test",
            bpm = 120
        )

        val track = Track(
            id = UUID.randomUUID(),
            project = project,
            name = "Guitar Track",
            instrument = "electric-guitar",
            trackNumber = 1,
            volume = 0.8f,
            pan = 0.5f
        )

        val response = TrackResponse.from(track)

        assertEquals(track.id, response.id)
        assertEquals(project.id, response.projectId)
        assertEquals("Guitar Track", response.name)
        assertEquals("electric-guitar", response.instrument)
        assertEquals(1, response.trackNumber)
        assertEquals(0.8f, response.volume)
        assertEquals(0.5f, response.pan)
        assertFalse(response.muted)
        assertFalse(response.soloed)
    }

    @Test
    fun `ErrorResponse should have timestamp`() {
        val error = ErrorResponse(
            message = "Test error",
            details = "Details here"
        )

        assertEquals("Test error", error.message)
        assertEquals("Details here", error.details)
        assertNotNull(error.timestamp)
        assertTrue(error.timestamp.isBefore(LocalDateTime.now().plusSeconds(1)))
    }

    @Test
    fun `RegisterRequest should validate email and password`() {
        val request = RegisterRequest(
            email = "test@example.com",
            username = "testuser",
            password = "password123"
        )

        assertEquals("test@example.com", request.email)
        assertEquals("testuser", request.username)
        assertEquals("password123", request.password)
    }

    @Test
    fun `LoginRequest should require email and password`() {
        val request = LoginRequest(
            email = "test@example.com",
            password = "password123"
        )

        assertEquals("test@example.com", request.email)
        assertEquals("password123", request.password)
    }

    @Test
    fun `AuthResponse should contain user info and token`() {
        val userId = UUID.randomUUID()
        val response = AuthResponse(
            token = "jwt.token.here",
            userId = userId,
            username = "testuser",
            email = "test@example.com"
        )

        assertEquals("jwt.token.here", response.token)
        assertEquals(userId, response.userId)
        assertEquals("testuser", response.username)
        assertEquals("test@example.com", response.email)
    }
}
