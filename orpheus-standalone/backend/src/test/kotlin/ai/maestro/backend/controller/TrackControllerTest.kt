package ai.maestro.backend.controller

import ai.maestro.backend.dto.*
import ai.maestro.backend.service.TrackService
import com.fasterxml.jackson.databind.ObjectMapper
import io.mockk.every
import io.mockk.mockk
import io.mockk.verify
import org.junit.jupiter.api.Test
import org.springframework.beans.factory.annotation.Autowired
import org.springframework.boot.test.autoconfigure.web.servlet.WebMvcTest
import org.springframework.boot.test.context.TestConfiguration
import org.springframework.context.annotation.Bean
import org.springframework.http.MediaType
import org.springframework.test.web.servlet.MockMvc
import org.springframework.test.web.servlet.request.MockMvcRequestBuilders.*
import org.springframework.test.web.servlet.result.MockMvcResultMatchers.*
import java.time.LocalDateTime
import java.util.UUID

@WebMvcTest(TrackController::class)
class TrackControllerTest {

    @Autowired
    private lateinit var mockMvc: MockMvc

    @Autowired
    private lateinit var trackService: TrackService

    @Autowired
    private lateinit var objectMapper: ObjectMapper

    @TestConfiguration
    class TestConfig {
        @Bean
        fun trackService(): TrackService = mockk(relaxed = true)
    }

    private val userId = UUID.randomUUID()
    private val projectId = UUID.randomUUID()
    private val trackId = UUID.randomUUID()

    @Test
    fun `POST projects projectId tracks should create track`() {
        val request = CreateTrackRequest(
            name = "Guitar Track",
            instrument = "electric-guitar",
            trackNumber = 1,
            volume = 0.8f,
            pan = 0.5f
        )

        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Guitar Track",
            instrument = "electric-guitar",
            trackNumber = 1,
            volume = 0.8f,
            pan = 0.5f,
            muted = false,
            soloed = false,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.createTrack(projectId, userId, request) } returns response

        mockMvc.perform(
            post("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isCreated)
            .andExpect(jsonPath("$.id").value(trackId.toString()))
            .andExpect(jsonPath("$.name").value("Guitar Track"))
            .andExpect(jsonPath("$.instrument").value("electric-guitar"))

        verify { trackService.createTrack(projectId, userId, request) }
    }

    @Test
    fun `POST projects projectId tracks should validate request`() {
        val invalidRequest = mapOf("invalid" to "data")

        mockMvc.perform(
            post("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(invalidRequest))
        )
            .andExpect(status().isBadRequest)
    }

    @Test
    fun `POST projects projectId tracks should return 404 when project not found`() {
        val request = CreateTrackRequest(
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        every { trackService.createTrack(projectId, userId, request) } throws
            IllegalArgumentException("Project not found")

        mockMvc.perform(
            post("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `POST projects projectId tracks should return 403 when no access`() {
        val request = CreateTrackRequest(
            name = "Track",
            instrument = "guitar",
            trackNumber = 1
        )

        every { trackService.createTrack(projectId, userId, request) } throws
            IllegalAccessException("No access to project")

        mockMvc.perform(
            post("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `GET projects projectId tracks should return all tracks`() {
        val tracks = listOf(
            TrackResponse(
                id = trackId,
                projectId = projectId,
                name = "Track 1",
                instrument = "guitar",
                trackNumber = 1,
                volume = 0.75f,
                pan = 0.5f,
                muted = false,
                soloed = false,
                created = LocalDateTime.now(),
                modified = LocalDateTime.now()
            ),
            TrackResponse(
                id = UUID.randomUUID(),
                projectId = projectId,
                name = "Track 2",
                instrument = "bass",
                trackNumber = 2,
                volume = 0.75f,
                pan = 0.5f,
                muted = false,
                soloed = false,
                created = LocalDateTime.now(),
                modified = LocalDateTime.now()
            )
        )

        every { trackService.getProjectTracks(projectId, userId) } returns tracks

        mockMvc.perform(
            get("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.length()").value(2))
            .andExpect(jsonPath("$[0].name").value("Track 1"))
            .andExpect(jsonPath("$[1].name").value("Track 2"))

        verify { trackService.getProjectTracks(projectId, userId) }
    }

    @Test
    fun `GET projects projectId tracks should return empty list when no tracks`() {
        every { trackService.getProjectTracks(projectId, userId) } returns emptyList()

        mockMvc.perform(
            get("/api/v1/projects/$projectId/tracks")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.length()").value(0))
    }

    @Test
    fun `GET tracks id should return track`() {
        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Guitar Track",
            instrument = "electric-guitar",
            trackNumber = 1,
            volume = 0.8f,
            pan = 0.5f,
            muted = false,
            soloed = false,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.getTrack(trackId, userId) } returns response

        mockMvc.perform(
            get("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.id").value(trackId.toString()))
            .andExpect(jsonPath("$.name").value("Guitar Track"))

        verify { trackService.getTrack(trackId, userId) }
    }

    @Test
    fun `GET tracks id should return 404 when not found`() {
        every { trackService.getTrack(trackId, userId) } throws
            IllegalArgumentException("Track not found")

        mockMvc.perform(
            get("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `GET tracks id should return 403 when no access`() {
        every { trackService.getTrack(trackId, userId) } throws
            IllegalAccessException("No access to track")

        mockMvc.perform(
            get("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `PUT tracks id should update track`() {
        val request = UpdateTrackRequest(
            name = "Updated Track",
            volume = 0.9f,
            muted = true
        )

        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Updated Track",
            instrument = "guitar",
            trackNumber = 1,
            volume = 0.9f,
            pan = 0.5f,
            muted = true,
            soloed = false,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.updateTrack(trackId, userId, request) } returns response

        mockMvc.perform(
            put("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.name").value("Updated Track"))
            .andExpect(jsonPath("$.volume").value(0.9))
            .andExpect(jsonPath("$.muted").value(true))

        verify { trackService.updateTrack(trackId, userId, request) }
    }

    @Test
    fun `PUT tracks id should allow partial updates`() {
        val request = UpdateTrackRequest(name = "Only Name")

        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Only Name",
            instrument = "guitar",
            trackNumber = 1,
            volume = 0.75f,
            pan = 0.5f,
            muted = false,
            soloed = false,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.updateTrack(trackId, userId, request) } returns response

        mockMvc.perform(
            put("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.name").value("Only Name"))
    }

    @Test
    fun `PUT tracks id should return 404 when not found`() {
        val request = UpdateTrackRequest(name = "Update")

        every { trackService.updateTrack(trackId, userId, request) } throws
            IllegalArgumentException("Track not found")

        mockMvc.perform(
            put("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `PUT tracks id should return 403 when no write access`() {
        val request = UpdateTrackRequest(name = "Update")

        every { trackService.updateTrack(trackId, userId, request) } throws
            IllegalAccessException("No write access")

        mockMvc.perform(
            put("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `DELETE tracks id should delete track`() {
        every { trackService.deleteTrack(trackId, userId) } returns Unit

        mockMvc.perform(
            delete("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNoContent)

        verify { trackService.deleteTrack(trackId, userId) }
    }

    @Test
    fun `DELETE tracks id should return 404 when not found`() {
        every { trackService.deleteTrack(trackId, userId) } throws
            IllegalArgumentException("Track not found")

        mockMvc.perform(
            delete("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `DELETE tracks id should return 403 when no write access`() {
        every { trackService.deleteTrack(trackId, userId) } throws
            IllegalAccessException("No write access")

        mockMvc.perform(
            delete("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `PUT tracks id audio should update audio file`() {
        val audioUrl = "https://example.com/audio.wav"

        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1,
            volume = 0.75f,
            pan = 0.5f,
            muted = false,
            soloed = false,
            audioFileUrl = audioUrl,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.updateAudioFile(trackId, userId, audioUrl) } returns response

        mockMvc.perform(
            put("/api/v1/tracks/$trackId/audio")
                .requestAttr("userId", userId)
                .param("audioUrl", audioUrl)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.audioFileUrl").value(audioUrl))

        verify { trackService.updateAudioFile(trackId, userId, audioUrl) }
    }

    @Test
    fun `PUT tracks id audio should return 404 when track not found`() {
        val audioUrl = "https://example.com/audio.wav"

        every { trackService.updateAudioFile(trackId, userId, audioUrl) } throws
            IllegalArgumentException("Track not found")

        mockMvc.perform(
            put("/api/v1/tracks/$trackId/audio")
                .requestAttr("userId", userId)
                .param("audioUrl", audioUrl)
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `PUT tracks id audio should return 403 when no write access`() {
        val audioUrl = "https://example.com/audio.wav"

        every { trackService.updateAudioFile(trackId, userId, audioUrl) } throws
            IllegalAccessException("No write access")

        mockMvc.perform(
            put("/api/v1/tracks/$trackId/audio")
                .requestAttr("userId", userId)
                .param("audioUrl", audioUrl)
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `should handle CORS headers`() {
        val response = TrackResponse(
            id = trackId,
            projectId = projectId,
            name = "Track",
            instrument = "guitar",
            trackNumber = 1,
            volume = 0.75f,
            pan = 0.5f,
            muted = false,
            soloed = false,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { trackService.getTrack(trackId, userId) } returns response

        mockMvc.perform(
            get("/api/v1/tracks/$trackId")
                .requestAttr("userId", userId)
                .header("Origin", "http://localhost:5173")
        )
            .andExpect(status().isOk)
    }

    @Test
    fun `should require userId attribute`() {
        mockMvc.perform(
            get("/api/v1/tracks/$trackId")
        )
            .andExpect(status().is4xxClientError)
    }
}
