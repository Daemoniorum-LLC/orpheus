package ai.maestro.backend.service

import ai.maestro.backend.dto.CreateTrackRequest
import ai.maestro.backend.dto.TrackResponse
import ai.maestro.backend.dto.UpdateTrackRequest
import ai.maestro.backend.model.Track
import ai.maestro.backend.repository.ProjectRepository
import ai.maestro.backend.repository.TrackRepository
import org.springframework.data.repository.findByIdOrNull
import org.springframework.stereotype.Service
import org.springframework.transaction.annotation.Transactional
import java.util.UUID

@Service
@Transactional
class TrackService(
    private val trackRepository: TrackRepository,
    private val projectRepository: ProjectRepository
) {

    fun createTrack(projectId: UUID, userId: UUID, request: CreateTrackRequest): TrackResponse {
        val project = projectRepository.findByIdOrNull(projectId)
            ?: throw IllegalArgumentException("Project not found")

        // Check write access
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have access to this project")
        }

        val track = Track(
            project = project,
            name = request.name,
            instrument = request.instrument,
            trackNumber = request.trackNumber,
            tablature = request.tablature,
            processors = request.processors,
            volume = request.volume,
            pan = request.pan
        )

        val saved = trackRepository.save(track)
        return TrackResponse.from(saved)
    }

    fun getTrack(trackId: UUID, userId: UUID): TrackResponse {
        val track = trackRepository.findByIdOrNull(trackId)
            ?: throw IllegalArgumentException("Track not found")

        // Check access
        val project = track.project
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have access to this track")
        }

        return TrackResponse.from(track)
    }

    fun getProjectTracks(projectId: UUID, userId: UUID): List<TrackResponse> {
        val project = projectRepository.findByIdOrNull(projectId)
            ?: throw IllegalArgumentException("Project not found")

        // Check access
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have access to this project")
        }

        return trackRepository.findByProjectOrderByTrackNumber(project)
            .map { TrackResponse.from(it) }
    }

    fun updateTrack(trackId: UUID, userId: UUID, request: UpdateTrackRequest): TrackResponse {
        val track = trackRepository.findByIdOrNull(trackId)
            ?: throw IllegalArgumentException("Track not found")

        // Check write access
        val project = track.project
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have write access to this track")
        }

        request.name?.let { track.name = it }
        request.instrument?.let { track.instrument = it }
        request.trackNumber?.let { track.trackNumber = it }
        request.tablature?.let { track.tablature = it }
        request.processors?.let { track.processors = it }
        request.volume?.let { track.volume = it }
        request.pan?.let { track.pan = it }
        request.muted?.let { track.muted = it }
        request.soloed?.let { track.soloed = it }

        val updated = trackRepository.save(track)
        return TrackResponse.from(updated)
    }

    fun deleteTrack(trackId: UUID, userId: UUID) {
        val track = trackRepository.findByIdOrNull(trackId)
            ?: throw IllegalArgumentException("Track not found")

        // Check write access
        val project = track.project
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have write access to this track")
        }

        trackRepository.delete(track)
    }

    fun updateAudioFile(trackId: UUID, userId: UUID, audioUrl: String): TrackResponse {
        val track = trackRepository.findByIdOrNull(trackId)
            ?: throw IllegalArgumentException("Track not found")

        // Check write access
        val project = track.project
        if (project.user.id != userId && !project.collaborators.any { it.user.id == userId }) {
            throw IllegalAccessException("You don't have write access to this track")
        }

        track.audioFileUrl = audioUrl
        val updated = trackRepository.save(track)
        return TrackResponse.from(updated)
    }
}
