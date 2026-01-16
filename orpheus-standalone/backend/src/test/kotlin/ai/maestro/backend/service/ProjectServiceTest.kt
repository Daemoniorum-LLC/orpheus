package ai.maestro.backend.service

import ai.maestro.backend.dto.CreateProjectRequest
import ai.maestro.backend.dto.UpdateProjectRequest
import ai.maestro.backend.model.Collaborator
import ai.maestro.backend.model.CollaboratorRole
import ai.maestro.backend.model.Project
import ai.maestro.backend.model.User
import ai.maestro.backend.repository.ProjectRepository
import ai.maestro.backend.repository.UserRepository
import io.mockk.*
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertThrows
import java.time.LocalDateTime
import java.util.UUID

class ProjectServiceTest {

    private lateinit var projectService: ProjectService
    private lateinit var projectRepository: ProjectRepository
    private lateinit var userRepository: UserRepository

    private lateinit var testUser: User
    private lateinit var testProject: Project
    private val userId = UUID.randomUUID()
    private val projectId = UUID.randomUUID()

    @BeforeEach
    fun setup() {
        projectRepository = mockk()
        userRepository = mockk()
        projectService = ProjectService(projectRepository, userRepository)

        testUser = User(
            id = userId,
            email = "test@example.com",
            username = "testuser",
            passwordHash = "hashedpassword"
        )

        testProject = Project(
            id = projectId,
            user = testUser,
            title = "Test Project",
            artist = "Test Artist",
            bpm = 120,
            timeSignature = "4/4",
            key = "C"
        )
    }

    @Test
    fun `createProject should create new project successfully`() {
        // Given
        val request = CreateProjectRequest(
            title = "New Project",
            artist = "Artist",
            bpm = 140,
            timeSignature = "4/4",
            key = "Gm"
        )

        every { userRepository.findByIdOrNull(userId) } returns testUser
        every { projectRepository.save(any()) } returns testProject

        // When
        val result = projectService.createProject(userId, request)

        // Then
        assertNotNull(result)
        assertEquals(testProject.title, result.title)
        verify { userRepository.findByIdOrNull(userId) }
        verify { projectRepository.save(any()) }
    }

    @Test
    fun `createProject should throw when user not found`() {
        // Given
        val request = CreateProjectRequest(title = "Test")
        every { userRepository.findByIdOrNull(userId) } returns null

        // When/Then
        assertThrows<IllegalArgumentException> {
            projectService.createProject(userId, request)
        }
    }

    @Test
    fun `getProject should return project when user has access`() {
        // Given
        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When
        val result = projectService.getProject(projectId, userId)

        // Then
        assertNotNull(result)
        assertEquals(testProject.title, result.title)
    }

    @Test
    fun `getProject should throw when project not found`() {
        // Given
        every { projectRepository.findByIdOrNull(projectId) } returns null

        // When/Then
        assertThrows<IllegalArgumentException> {
            projectService.getProject(projectId, userId)
        }
    }

    @Test
    fun `getProject should throw when user has no access`() {
        // Given
        val otherUserId = UUID.randomUUID()
        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When/Then
        assertThrows<IllegalAccessException> {
            projectService.getProject(projectId, otherUserId)
        }
    }

    @Test
    fun `getAllProjects should return all accessible projects`() {
        // Given
        val projects = listOf(testProject)
        every { projectRepository.findAllAccessibleByUserId(userId) } returns projects

        // When
        val result = projectService.getAllProjects(userId)

        // Then
        assertEquals(1, result.size)
        assertEquals(testProject.title, result[0].title)
    }

    @Test
    fun `updateProject should update project successfully`() {
        // Given
        val request = UpdateProjectRequest(
            title = "Updated Title",
            bpm = 150
        )

        every { projectRepository.findByIdOrNull(projectId) } returns testProject
        every { projectRepository.save(any()) } returns testProject

        // When
        val result = projectService.updateProject(projectId, userId, request)

        // Then
        assertNotNull(result)
        verify { projectRepository.save(any()) }
    }

    @Test
    fun `updateProject should throw when user has no write access`() {
        // Given
        val otherUserId = UUID.randomUUID()
        val request = UpdateProjectRequest(title = "Test")

        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When/Then
        assertThrows<IllegalAccessException> {
            projectService.updateProject(projectId, otherUserId, request)
        }
    }

    @Test
    fun `deleteProject should delete project when user is owner`() {
        // Given
        every { projectRepository.findByIdOrNull(projectId) } returns testProject
        every { projectRepository.delete(testProject) } just Runs

        // When
        projectService.deleteProject(projectId, userId)

        // Then
        verify { projectRepository.delete(testProject) }
    }

    @Test
    fun `deleteProject should throw when user is not owner`() {
        // Given
        val otherUserId = UUID.randomUUID()
        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When/Then
        assertThrows<IllegalAccessException> {
            projectService.deleteProject(projectId, otherUserId)
        }
    }

    @Test
    fun `collaborator should have read access`() {
        // Given
        val collaboratorUserId = UUID.randomUUID()
        val collaboratorUser = User(
            id = collaboratorUserId,
            email = "collab@example.com",
            username = "collaborator",
            passwordHash = "hash"
        )

        testProject.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = testProject,
                user = collaboratorUser,
                role = CollaboratorRole.VIEWER,
                addedAt = LocalDateTime.now()
            )
        )

        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When
        val result = projectService.getProject(projectId, collaboratorUserId)

        // Then
        assertNotNull(result)
    }

    @Test
    fun `editor should have write access`() {
        // Given
        val editorUserId = UUID.randomUUID()
        val editorUser = User(
            id = editorUserId,
            email = "editor@example.com",
            username = "editor",
            passwordHash = "hash"
        )

        testProject.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = testProject,
                user = editorUser,
                role = CollaboratorRole.EDITOR,
                addedAt = LocalDateTime.now()
            )
        )

        val request = UpdateProjectRequest(title = "Updated")

        every { projectRepository.findByIdOrNull(projectId) } returns testProject
        every { projectRepository.save(any()) } returns testProject

        // When
        val result = projectService.updateProject(projectId, editorUserId, request)

        // Then
        assertNotNull(result)
        verify { projectRepository.save(any()) }
    }

    @Test
    fun `viewer should not have write access`() {
        // Given
        val viewerUserId = UUID.randomUUID()
        val viewerUser = User(
            id = viewerUserId,
            email = "viewer@example.com",
            username = "viewer",
            passwordHash = "hash"
        )

        testProject.collaborators.add(
            Collaborator(
                id = UUID.randomUUID(),
                project = testProject,
                user = viewerUser,
                role = CollaboratorRole.VIEWER,
                addedAt = LocalDateTime.now()
            )
        )

        val request = UpdateProjectRequest(title = "Updated")

        every { projectRepository.findByIdOrNull(projectId) } returns testProject

        // When/Then
        assertThrows<IllegalAccessException> {
            projectService.updateProject(projectId, viewerUserId, request)
        }
    }
}
