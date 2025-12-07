package ai.maestro.backend.controller

import ai.maestro.backend.dto.*
import ai.maestro.backend.service.TrackService
import jakarta.validation.Valid
import org.springframework.http.HttpStatus
import org.springframework.http.ResponseEntity
import org.springframework.web.bind.annotation.*
import java.util.UUID

@RestController
@RequestMapping("/api/v1")
@CrossOrigin(origins = ["http://localhost:5173", "http://localhost:3000"])
class TrackController(
    private val trackService: TrackService
) {

    @PostMapping("/projects/{projectId}/tracks")
    fun createTrack(
        @PathVariable projectId: UUID,
        @RequestAttribute("userId") userId: UUID,
        @Valid @RequestBody request: CreateTrackRequest
    ): ResponseEntity<TrackResponse> {
        val track = trackService.createTrack(projectId, userId, request)
        return ResponseEntity.status(HttpStatus.CREATED).body(track)
    }

    @GetMapping("/projects/{projectId}/tracks")
    fun getProjectTracks(
        @PathVariable projectId: UUID,
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<List<TrackResponse>> {
        val tracks = trackService.getProjectTracks(projectId, userId)
        return ResponseEntity.ok(tracks)
    }

    @GetMapping("/tracks/{id}")
    fun getTrack(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<TrackResponse> {
        val track = trackService.getTrack(id, userId)
        return ResponseEntity.ok(track)
    }

    @PutMapping("/tracks/{id}")
    fun updateTrack(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID,
        @Valid @RequestBody request: UpdateTrackRequest
    ): ResponseEntity<TrackResponse> {
        val track = trackService.updateTrack(id, userId, request)
        return ResponseEntity.ok(track)
    }

    @DeleteMapping("/tracks/{id}")
    fun deleteTrack(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID
    ): ResponseEntity<Void> {
        trackService.deleteTrack(id, userId)
        return ResponseEntity.noContent().build()
    }

    @PutMapping("/tracks/{id}/audio")
    fun updateAudioFile(
        @PathVariable id: UUID,
        @RequestAttribute("userId") userId: UUID,
        @RequestParam audioUrl: String
    ): ResponseEntity<TrackResponse> {
        val track = trackService.updateAudioFile(id, userId, audioUrl)
        return ResponseEntity.ok(track)
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
