package ai.maestro.backend.service

import ai.maestro.backend.dto.CreateTrackRequest
import ai.maestro.backend.dto.UpdateTrackRequest
import ai.maestro.backend.model.Collaborator
import ai.maestro.backend.model.CollaboratorRole
import ai.maestro.backend.model.Project
import ai.maestro.backend.model.Track
import ai.maestro.backend.model.User
import ai.maestro.backend.repository.ProjectRepository
import ai.maestro.backend.repository.TrackRepository
import io.mockk.*
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import org.springframework.data.repository.findByIdOrNull
import java.util.UUID

class TrackServiceTest {

    private lateinit var trackService: TrackService
    private lateinit var trackRepository: TrackRepository
    private lateinit var projectRepository: ProjectRepository

    private lateinit var owner: User
    private lateinit var editor: User
    private lateinit var viewer: User
    private lateinit var project: Project
    private lateinit var track: Track

    @BeforeEach
    fun setup() {
        trackRepository = mockk()
        projectRepository = mockk()
        trackService = TrackService(trackRepository, projectRepository)

        owner = User(
            id = UUID.randomUUID(),
            email = "owner@test.com",
            username = "owner",
            passwordHash = "hash"
        )

        editor = User(
            id = UUID.randomUUID(),
            email = "editor@test.com",
            username = "editor",
            passwordHash = "hash"
        )

        viewer = User(
            id = UUID.randomUUID(),
            email = "viewer@test.com",
            username = "viewer",
            passwordHash = "hash"
        )

        project = Project(
            id = UUID.randomUUID(),
            user = owner,
            title = "Test Project",
            bpm = 120
        )

        track = Track(
            id = UUID.randomUUID(),
            project = project,
            name = "Test Track",
            instrument = "guitar",
            trackNumber = 1
        )
    }

    @Test
    fun `createTrack should create track for project owner`() {
        val request = CreateTrackRequest(
            name = "New Track",
            instrument = "bass",
            trackNumber = 1,
            volume = 0.8f,
            pan = 0.5f
        )

        every { projectRepository.findByIdOrNull(project.id) } returns project
        every { trackRepository.save(any()) } returns track

        val result = trackService.createTrack(project.id!!, owner.id!!, request)

        assertNotNull(result)
        verify { trackRepository.save(any()) }
    }

    @Test
    fun `createTrack should create track for collaborator`() {
        project.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = project,
                user = editor,
                role = CollaboratorRole.EDITOR
            )
        )

        val request = CreateTrackRequest(
            name = "Track",
            instrument = "drums",
            trackNumber = 1
        )

        every { projectRepository.findByIdOrNull(project.id) } returns project
        every { trackRepository.save(any()) } returns track

        val result = trackService.createTrack(project.id!!, editor.id!!, request)

        assertNotNull(result)
        verify { trackRepository.save(any()) }
    }

    @Test
    fun `createTrack should throw when project not found`() {
        val request = CreateTrackRequest(
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        every { projectRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.createTrack(UUID.randomUUID(), owner.id!!, request)
        }
    }

    @Test
    fun `createTrack should throw when no access`() {
        val request = CreateTrackRequest(
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        every { projectRepository.findByIdOrNull(project.id) } returns project

        assertThrows<IllegalAccessException> {
            trackService.createTrack(project.id!!, UUID.randomUUID(), request)
        }
    }

    @Test
    fun `createTrack should apply default values`() {
        val request = CreateTrackRequest(
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        every { projectRepository.findByIdOrNull(project.id) } returns project
        every { trackRepository.save(any()) } answers {
            val saved = firstArg<Track>()
            assertEquals(0.75f, saved.volume)
            assertEquals(0.5f, saved.pan)
            saved
        }

        trackService.createTrack(project.id!!, owner.id!!, request)

        verify { trackRepository.save(any()) }
    }

    @Test
    fun `getTrack should return track for owner`() {
        every { trackRepository.findByIdOrNull(track.id) } returns track

        val result = trackService.getTrack(track.id!!, owner.id!!)

        assertNotNull(result)
        assertEquals(track.id, result.id)
    }

    @Test
    fun `getTrack should return track for collaborator`() {
        project.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = project,
                user = viewer,
                role = CollaboratorRole.VIEWER
            )
        )

        every { trackRepository.findByIdOrNull(track.id) } returns track

        val result = trackService.getTrack(track.id!!, viewer.id!!)

        assertNotNull(result)
    }

    @Test
    fun `getTrack should throw when track not found`() {
        every { trackRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.getTrack(UUID.randomUUID(), owner.id!!)
        }
    }

    @Test
    fun `getTrack should throw when no access`() {
        every { trackRepository.findByIdOrNull(track.id) } returns track

        assertThrows<IllegalAccessException> {
            trackService.getTrack(track.id!!, UUID.randomUUID())
        }
    }

    @Test
    fun `getProjectTracks should return all tracks ordered by number`() {
        val track1 = Track(
            id = UUID.randomUUID(),
            project = project,
            name = "Track 1",
            instrument = "guitar",
            trackNumber = 1
        )

        val track2 = Track(
            id = UUID.randomUUID(),
            project = project,
            name = "Track 2",
            instrument = "bass",
            trackNumber = 2
        )

        every { projectRepository.findByIdOrNull(project.id) } returns project
        every { trackRepository.findByProjectOrderByTrackNumber(project) } returns listOf(track1, track2)

        val result = trackService.getProjectTracks(project.id!!, owner.id!!)

        assertEquals(2, result.size)
        assertEquals("Track 1", result[0].name)
        assertEquals("Track 2", result[1].name)
    }

    @Test
    fun `getProjectTracks should return empty list when no tracks`() {
        every { projectRepository.findByIdOrNull(project.id) } returns project
        every { trackRepository.findByProjectOrderByTrackNumber(project) } returns emptyList()

        val result = trackService.getProjectTracks(project.id!!, owner.id!!)

        assertEquals(0, result.size)
    }

    @Test
    fun `getProjectTracks should throw when project not found`() {
        every { projectRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.getProjectTracks(UUID.randomUUID(), owner.id!!)
        }
    }

    @Test
    fun `getProjectTracks should throw when no access`() {
        every { projectRepository.findByIdOrNull(project.id) } returns project

        assertThrows<IllegalAccessException> {
            trackService.getProjectTracks(project.id!!, UUID.randomUUID())
        }
    }

    @Test
    fun `updateTrack should update track for owner`() {
        val request = UpdateTrackRequest(
            name = "Updated Track",
            volume = 0.9f
        )

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.save(any()) } returns track

        val result = trackService.updateTrack(track.id!!, owner.id!!, request)

        assertNotNull(result)
        verify { trackRepository.save(any()) }
    }

    @Test
    fun `updateTrack should update track for collaborator`() {
        project.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = project,
                user = editor,
                role = CollaboratorRole.EDITOR
            )
        )

        val request = UpdateTrackRequest(name = "Updated")

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.save(any()) } returns track

        val result = trackService.updateTrack(track.id!!, editor.id!!, request)

        assertNotNull(result)
    }

    @Test
    fun `updateTrack should apply partial updates`() {
        val request = UpdateTrackRequest(
            name = "New Name",
            volume = 0.9f
        )

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.save(any()) } answers {
            val updated = firstArg<Track>()
            assertEquals("New Name", updated.name)
            assertEquals(0.9f, updated.volume)
            updated
        }

        trackService.updateTrack(track.id!!, owner.id!!, request)

        verify { trackRepository.save(any()) }
    }

    @Test
    fun `updateTrack should throw when track not found`() {
        val request = UpdateTrackRequest(name = "Update")

        every { trackRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.updateTrack(UUID.randomUUID(), owner.id!!, request)
        }
    }

    @Test
    fun `updateTrack should throw when no write access`() {
        val request = UpdateTrackRequest(name = "Update")

        every { trackRepository.findByIdOrNull(track.id) } returns track

        assertThrows<IllegalAccessException> {
            trackService.updateTrack(track.id!!, UUID.randomUUID(), request)
        }
    }

    @Test
    fun `deleteTrack should delete track for owner`() {
        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.delete(track) } just Runs

        trackService.deleteTrack(track.id!!, owner.id!!)

        verify { trackRepository.delete(track) }
    }

    @Test
    fun `deleteTrack should delete track for collaborator`() {
        project.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = project,
                user = editor,
                role = CollaboratorRole.EDITOR
            )
        )

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.delete(track) } just Runs

        trackService.deleteTrack(track.id!!, editor.id!!)

        verify { trackRepository.delete(track) }
    }

    @Test
    fun `deleteTrack should throw when track not found`() {
        every { trackRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.deleteTrack(UUID.randomUUID(), owner.id!!)
        }
    }

    @Test
    fun `deleteTrack should throw when no write access`() {
        every { trackRepository.findByIdOrNull(track.id) } returns track

        assertThrows<IllegalAccessException> {
            trackService.deleteTrack(track.id!!, UUID.randomUUID())
        }
    }

    @Test
    fun `updateAudioFile should update audio URL for owner`() {
        val audioUrl = "https://example.com/audio.wav"

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.save(any()) } returns track

        val result = trackService.updateAudioFile(track.id!!, owner.id!!, audioUrl)

        assertNotNull(result)
        verify { trackRepository.save(any()) }
    }

    @Test
    fun `updateAudioFile should update audio URL for collaborator`() {
        project.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = project,
                user = editor,
                role = CollaboratorRole.EDITOR
            )
        )

        val audioUrl = "https://example.com/audio.wav"

        every { trackRepository.findByIdOrNull(track.id) } returns track
        every { trackRepository.save(any()) } returns track

        val result = trackService.updateAudioFile(track.id!!, editor.id!!, audioUrl)

        assertNotNull(result)
    }

    @Test
    fun `updateAudioFile should throw when track not found`() {
        every { trackRepository.findByIdOrNull(any()) } returns null

        assertThrows<IllegalArgumentException> {
            trackService.updateAudioFile(UUID.randomUUID(), owner.id!!, "url")
        }
    }

    @Test
    fun `updateAudioFile should throw when no write access`() {
        every { trackRepository.findByIdOrNull(track.id) } returns track

        assertThrows<IllegalAccessException> {
            trackService.updateAudioFile(track.id!!, UUID.randomUUID(), "url")
        }
    }
}
