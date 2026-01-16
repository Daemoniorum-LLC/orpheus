package ai.maestro.backend.service

import ai.maestro.backend.dto.*
import ai.maestro.backend.model.CollaboratorRole
import ai.maestro.backend.model.Project
import ai.maestro.backend.model.User
import ai.maestro.backend.repository.ProjectRepository
import ai.maestro.backend.repository.UserRepository
import org.springframework.data.repository.findByIdOrNull
import org.springframework.stereotype.Service
import org.springframework.transaction.annotation.Transactional
import java.util.UUID

@Service
@Transactional
class ProjectService(
    private val projectRepository: ProjectRepository,
    private val userRepository: UserRepository
) {

    fun createProject(userId: UUID, request: CreateProjectRequest): ProjectResponse {
        val user = userRepository.findByIdOrNull(userId)
            ?: throw IllegalArgumentException("User not found")

        val project = Project(
            user = user,
            title = request.title,
            artist = request.artist,
            bpm = request.bpm,
            timeSignature = request.timeSignature,
            key = request.key,
            metadata = request.metadata
        )

        val saved = projectRepository.save(project)
        return ProjectResponse.from(saved)
    }

    fun getProject(projectId: UUID, userId: UUID): ProjectDetailResponse {
        val project = projectRepository.findByIdOrNull(projectId)
            ?: throw IllegalArgumentException("Project not found")

        // Check access
        if (!hasAccess(project, userId)) {
            throw IllegalAccessException("You don't have access to this project")
        }

        return ProjectDetailResponse.from(project)
    }

    fun getAllProjects(userId: UUID): List<ProjectResponse> {
        return projectRepository.findAllAccessibleByUserId(userId)
            .map { ProjectResponse.from(it) }
    }

    fun updateProject(projectId: UUID, userId: UUID, request: UpdateProjectRequest): ProjectResponse {
        val project = projectRepository.findByIdOrNull(projectId)
            ?: throw IllegalArgumentException("Project not found")

        // Check write access
        if (!hasWriteAccess(project, userId)) {
            throw IllegalAccessException("You don't have write access to this project")
        }

        request.title?.let { project.title = it }
        request.artist?.let { project.artist = it }
        request.bpm?.let { project.bpm = it }
        request.timeSignature?.let { project.timeSignature = it }
        request.key?.let { project.key = it }
        request.metadata?.let { project.metadata = it }

        val updated = projectRepository.save(project)
        return ProjectResponse.from(updated)
    }

    fun deleteProject(projectId: UUID, userId: UUID) {
        val project = projectRepository.findByIdOrNull(projectId)
            ?: throw IllegalArgumentException("Project not found")

        // Only owner can delete
        if (project.user.id != userId) {
            throw IllegalAccessException("Only the project owner can delete it")
        }

        projectRepository.delete(project)
    }

    private fun hasAccess(project: Project, userId: UUID): Boolean {
        // Owner always has access
        if (project.user.id == userId) return true

        // Check if user is a collaborator
        return project.collaborators.any { it.user.id == userId }
    }

    private fun hasWriteAccess(project: Project, userId: UUID): Boolean {
        // Owner always has write access
        if (project.user.id == userId) return true

        // Check if user is an owner or editor collaborator
        return project.collaborators.any {
            it.user.id == userId && it.role in listOf(CollaboratorRole.OWNER, CollaboratorRole.EDITOR)
        }
    }
}
