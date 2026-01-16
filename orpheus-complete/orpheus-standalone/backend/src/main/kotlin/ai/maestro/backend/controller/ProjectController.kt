package ai.maestro.backend.controller

import ai.maestro.backend.dto.*
import ai.maestro.backend.service.ProjectService
import jakarta.validation.Valid
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.*
import java.util.UUID

@RestController
@RequestMapping("/api/v1/projects")
@CrossOrigin(origins = ["http://localhost:5173", "http://localhost:3000"])
class ProjectController(
    private val projectService: ProjectService
) {

    @PostMapping
    fun createProject(
        @RequestAttribute("userId") userId: UUID,
        @Valid @RequestBody request: CreateProjectRequest
    ): ResponseEntity<ProjectResponse> {
        val project = projectService.createProject(userId, request)
        return ResponseEntity.status(HttpStatus.CREATED).body(project)
    }

    @GetMapping
    fun getAllProjects(
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<List<ProjectResponse>> {
        val projects = projectService.getAllProjects(userId)
        return ResponseEntity.ok(projects)
    }

    @GetMapping("/{id}")
    fun getProject(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<ProjectDetailResponse> {
        val project = projectService.getProject(id, userId)
        return ResponseEntity.ok(project)
    }

    @PutMapping("/{id}")
    fun updateProject(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID,
        @Valid @RequestBody request: UpdateProjectRequest
    ): ResponseEntity<ProjectResponse> {
        val project = projectService.updateProject(id, userId, request)
        return ResponseEntity.ok(project)
    }

    @DeleteMapping("/{id}")
    fun deleteProject(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<Void> {
        projectService.deleteProject(id, userId)
        return ResponseEntity.noContent().build()
    }

    @ExceptionHandler(IllegalArgumentException::class)
    fun handleIllegalArgument(e: IllegalArgumentException): ResponseEntity<ErrorResponse> {
        return ResponseEntity
            .status(HttpStatus.NOT_FOUND)
            .body(ErrorResponse(message = e.message ?: "Resource not found"))
    }

    @ExceptionHandler(IllegalAccessException::class)
    fun handleIllegalAccess(e: IllegalAccessException): ResponseEntity<ErrorResponse> {
        return ResponseEntity
            .status(HttpStatus.FORBIDDEN)
            .body(ErrorResponse(message = e.message ?: "Access denied"))
    }
}
