package ai.maestro.backend.controller

import ai.maestro.backend.dto.*
import ai.maestro.backend.service.ProjectService
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

@WebMvcTest(ProjectController::class)
class ProjectControllerTest {

    @Autowired
    private lateinit var mockMvc: MockMvc

    @Autowired
    private lateinit var projectService: ProjectService

    @Autowired
    private lateinit var objectMapper: ObjectMapper

    @TestConfiguration
    class TestConfig {
        @Bean
        fun projectService(): ProjectService = mockk(relaxed = true)
    }

    private val userId = UUID.randomUUID()
    private val projectId = UUID.randomUUID()

    @Test
    fun `POST projects should create project`() {
        val request = CreateProjectRequest(
            title = "Test Project",
            bpm = 120,
            timeSignature = "4/4"
        )

        val response = ProjectResponse(
            id = projectId,
            userId = userId,
            title = "Test Project",
            bpm = 120,
            timeSignature = "4/4",
            trackCount = 0,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { projectService.createProject(userId, request) } returns response

        mockMvc.perform(
            post("/api/v1/projects")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isCreated)
            .andExpect(jsonPath("$.id").value(projectId.toString()))
            .andExpect(jsonPath("$.title").value("Test Project"))
            .andExpect(jsonPath("$.bpm").value(120))

        verify { projectService.createProject(userId, request) }
    }

    @Test
    fun `POST projects should validate request body`() {
        val invalidRequest = mapOf("invalid" to "data")

        mockMvc.perform(
            post("/api/v1/projects")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(invalidRequest))
        )
            .andExpect(status().isBadRequest)
    }

    @Test
    fun `GET projects should return all user projects`() {
        val projects = listOf(
            ProjectResponse(
                id = projectId,
                userId = userId,
                title = "Project 1",
                bpm = 120,
                timeSignature = "4/4",
                trackCount = 2,
                created = LocalDateTime.now(),
                modified = LocalDateTime.now()
            ),
            ProjectResponse(
                id = UUID.randomUUID(),
                userId = userId,
                title = "Project 2",
                bpm = 140,
                timeSignature = "3/4",
                trackCount = 1,
                created = LocalDateTime.now(),
                modified = LocalDateTime.now()
            )
        )

        every { projectService.getAllProjects(userId) } returns projects

        mockMvc.perform(
            get("/api/v1/projects")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.length()").value(2))
            .andExpect(jsonPath("$[0].title").value("Project 1"))
            .andExpect(jsonPath("$[1].title").value("Project 2"))

        verify { projectService.getAllProjects(userId) }
    }

    @Test
    fun `GET projects should return empty list when no projects`() {
        every { projectService.getAllProjects(userId) } returns emptyList()

        mockMvc.perform(
            get("/api/v1/projects")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.length()").value(0))
    }

    @Test
    fun `GET projects id should return project details`() {
        val response = ProjectDetailResponse(
            id = projectId,
            userId = userId,
            title = "Test Project",
            artist = "Test Artist",
            bpm = 120,
            timeSignature = "4/4",
            key = "C",
            tracks = listOf(
                TrackResponse(
                    id = UUID.randomUUID(),
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
                )
            ),
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { projectService.getProject(projectId, userId) } returns response

        mockMvc.perform(
            get("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.id").value(projectId.toString()))
            .andExpect(jsonPath("$.title").value("Test Project"))
            .andExpect(jsonPath("$.tracks.length()").value(1))

        verify { projectService.getProject(projectId, userId) }
    }

    @Test
    fun `GET projects id should return 404 when not found`() {
        every { projectService.getProject(projectId, userId) } throws
            IllegalArgumentException("Project not found")

        mockMvc.perform(
            get("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNotFound)
            .andExpect(jsonPath("$.message").value("Project not found"))
    }

    @Test
    fun `GET projects id should return 403 when access denied`() {
        every { projectService.getProject(projectId, userId) } throws
            IllegalAccessException("Access denied")

        mockMvc.perform(
            get("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isForbidden)
            .andExpect(jsonPath("$.message").value("Access denied"))
    }

    @Test
    fun `PUT projects id should update project`() {
        val request = UpdateProjectRequest(
            title = "Updated Title",
            bpm = 140
        )

        val response = ProjectResponse(
            id = projectId,
            userId = userId,
            title = "Updated Title",
            bpm = 140,
            timeSignature = "4/4",
            trackCount = 0,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { projectService.updateProject(projectId, userId, request) } returns response

        mockMvc.perform(
            put("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.title").value("Updated Title"))
            .andExpect(jsonPath("$.bpm").value(140))

        verify { projectService.updateProject(projectId, userId, request) }
    }

    @Test
    fun `PUT projects id should allow partial updates`() {
        val request = UpdateProjectRequest(title = "Only Title Updated")

        val response = ProjectResponse(
            id = projectId,
            userId = userId,
            title = "Only Title Updated",
            bpm = 120,
            timeSignature = "4/4",
            trackCount = 0,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { projectService.updateProject(projectId, userId, request) } returns response

        mockMvc.perform(
            put("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isOk)
            .andExpect(jsonPath("$.title").value("Only Title Updated"))
    }

    @Test
    fun `PUT projects id should return 404 when not found`() {
        val request = UpdateProjectRequest(title = "Update")

        every { projectService.updateProject(projectId, userId, request) } throws
            IllegalArgumentException("Project not found")

        mockMvc.perform(
            put("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `PUT projects id should return 403 when no write access`() {
        val request = UpdateProjectRequest(title = "Update")

        every { projectService.updateProject(projectId, userId, request) } throws
            IllegalAccessException("No write access")

        mockMvc.perform(
            put("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request))
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `DELETE projects id should delete project`() {
        every { projectService.deleteProject(projectId, userId) } returns Unit

        mockMvc.perform(
            delete("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNoContent)

        verify { projectService.deleteProject(projectId, userId) }
    }

    @Test
    fun `DELETE projects id should return 404 when not found`() {
        every { projectService.deleteProject(projectId, userId) } throws
            IllegalArgumentException("Project not found")

        mockMvc.perform(
            delete("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isNotFound)
    }

    @Test
    fun `DELETE projects id should return 403 when not owner`() {
        every { projectService.deleteProject(projectId, userId) } throws
            IllegalAccessException("Only project owner can delete")

        mockMvc.perform(
            delete("/api/v1/projects/$projectId")
                .requestAttr("userId", userId)
        )
            .andExpect(status().isForbidden)
    }

    @Test
    fun `should handle CORS headers`() {
        val response = ProjectResponse(
            id = projectId,
            userId = userId,
            title = "Test",
            bpm = 120,
            timeSignature = "4/4",
            trackCount = 0,
            created = LocalDateTime.now(),
            modified = LocalDateTime.now()
        )

        every { projectService.getAllProjects(userId) } returns listOf(response)

        mockMvc.perform(
            get("/api/v1/projects")
                .requestAttr("userId", userId)
                .header("Origin", "http://localhost:5173")
        )
            .andExpect(status().isOk)
    }

    @Test
    fun `should require userId attribute`() {
        mockMvc.perform(
            get("/api/v1/projects")
        )
            .andExpect(status().is4xxClientError)
    }
}
