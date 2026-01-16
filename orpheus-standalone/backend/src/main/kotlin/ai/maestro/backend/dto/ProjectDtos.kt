package ai.maestro.backend.dto

import ai.maestro.backend.model.DistributionStatus
import ai.maestro.backend.model.Project
import ai.maestro.backend.model.Track
import ai.maestro.backend.model.Distribution
import java.time.LocalDateTime
import java.util.UUID

// Project DTOs
data class CreateProjectRequest(
    val title: String,
    val artist: String? = null,
    val bpm: Int = 120,
    val timeSignature: String = "4/4",
    val key: String? = null,
    val metadata: Map<String, Any>? = null
)

data class UpdateProjectRequest(
    val title: String? = null,
    val artist: String? = null,
    val bpm: Int? = null,
    val timeSignature: String? = null,
    val key: String? = null,
    val metadata: Map<String, Any>? = null
)

data class ProjectResponse(
    val id: UUID,
    val userId: UUID,
    val title: String,
    val artist: String?,
    val bpm: Int,
    val timeSignature: String,
    val key: String?,
    val metadata: Map<String, Any>?,
    val trackCount: Int,
    val createdAt: LocalDateTime,
    val updatedAt: LocalDateTime
) {
    companion object {
        fun from(project: Project): ProjectResponse {
            return ProjectResponse(
                id = project.id,
                userId = project.user.id,
                title = project.title,
                artist = project.artist,
                bpm = project.bpm,
                timeSignature = project.timeSignature,
                key = project.key,
                metadata = project.metadata,
                trackCount = project.tracks.size,
                createdAt = project.createdAt,
                updatedAt = project.updatedAt
            )
        }
    }
}

data class ProjectDetailResponse(
    val id: UUID,
    val userId: UUID,
    val title: String,
    val artist: String?,
    val bpm: Int,
    val timeSignature: String,
    val key: String?,
    val metadata: Map<String, Any>?,
    val tracks: List<TrackResponse>,
    val createdAt: LocalDateTime,
    val updatedAt: LocalDateTime
) {
    companion object {
        fun from(project: Project): ProjectDetailResponse {
            return ProjectDetailResponse(
                id = project.id,
                userId = project.user.id,
                title = project.title,
                artist = project.artist,
                bpm = project.bpm,
                timeSignature = project.timeSignature,
                key = project.key,
                metadata = project.metadata,
                tracks = project.tracks.map { TrackResponse.from(it) },
                createdAt = project.createdAt,
                updatedAt = project.updatedAt
            )
        }
    }
}

// Track DTOs
data class CreateTrackRequest(
    val name: String,
    val instrument: String,
    val trackNumber: Int,
    val tablature: Map<String, Any>? = null,
    val processors: Map<String, Any>? = null,
    val volume: Float = 0.75f,
    val pan: Float = 0.5f
)

data class UpdateTrackRequest(
    val name: String? = null,
    val instrument: String? = null,
    val trackNumber: Int? = null,
    val tablature: Map<String, Any>? = null,
    val processors: Map<String, Any>? = null,
    val volume: Float? = null,
    val pan: Float? = null,
    val muted: Boolean? = null,
    val soloed: Boolean? = null
)

data class TrackResponse(
    val id: UUID,
    val projectId: UUID,
    val name: String,
    val instrument: String,
    val trackNumber: Int,
    val tablature: Map<String, Any>?,
    val audioFileUrl: String?,
    val processors: Map<String, Any>?,
    val volume: Float,
    val pan: Float,
    val muted: Boolean,
    val soloed: Boolean,
    val createdAt: LocalDateTime,
    val updatedAt: LocalDateTime
) {
    companion object {
        fun from(track: Track): TrackResponse {
            return TrackResponse(
                id = track.id,
                projectId = track.project.id,
                name = track.name,
                instrument = track.instrument,
                trackNumber = track.trackNumber,
                tablature = track.tablature,
                audioFileUrl = track.audioFileUrl,
                processors = track.processors,
                volume = track.volume,
                pan = track.pan,
                muted = track.muted,
                soloed = track.soloed,
                createdAt = track.createdAt,
                updatedAt = track.updatedAt
            )
        }
    }
}

// Distribution DTOs
data class CreateDistributionRequest(
    val title: String,
    val artist: String,
    val album: String? = null,
    val genre: String? = null,
    val isrc: String? = null,
    val upc: String? = null,
    val platforms: List<String>? = null
)

data class UpdateDistributionRequest(
    val title: String? = null,
    val artist: String? = null,
    val album: String? = null,
    val genre: String? = null,
    val isrc: String? = null,
    val upc: String? = null,
    val platforms: List<String>? = null,
    val status: DistributionStatus? = null
)

data class DistributionResponse(
    val id: UUID,
    val projectId: UUID,
    val title: String,
    val artist: String,
    val album: String?,
    val genre: String?,
    val isrc: String?,
    val upc: String?,
    val artworkUrl: String?,
    val platforms: List<String>?,
    val status: DistributionStatus,
    val submittedAt: LocalDateTime?,
    val publishedAt: LocalDateTime?,
    val createdAt: LocalDateTime,
    val updatedAt: LocalDateTime
) {
    companion object {
        fun from(distribution: Distribution): DistributionResponse {
            return DistributionResponse(
                id = distribution.id,
                projectId = distribution.project.id,
                title = distribution.title,
                artist = distribution.artist,
                album = distribution.album,
                genre = distribution.genre,
                isrc = distribution.isrc,
                upc = distribution.upc,
                artworkUrl = distribution.artworkUrl,
                platforms = distribution.platforms,
                status = distribution.status,
                submittedAt = distribution.submittedAt,
                publishedAt = distribution.publishedAt,
                createdAt = distribution.createdAt,
                updatedAt = distribution.updatedAt
            )
        }
    }
}

// Authentication DTOs
data class RegisterRequest(
    val email: String,
    val username: String,
    val password: String
)

data class LoginRequest(
    val email: String,
    val password: String
)

data class AuthResponse(
    val token: String,
    val userId: UUID,
    val username: String,
    val email: String
)

// Error response
data class ErrorResponse(
    val message: String,
    val details: String? = null,
    val timestamp: LocalDateTime = LocalDateTime.now()
)
