package ai.maestro.backend.repository

import ai.maestro.backend.model.Project
import ai.maestro.backend.model.Track
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.stereotype.Repository
import java.util.UUID

@Repository
interface TrackRepository : JpaRepository<Track, UUID> {
    fun findByProject(project: Project): List<Track>
    fun findByProjectOrderByTrackNumber(project: Project): List<Track>
    fun findByProjectIdOrderByTrackNumber(projectId: UUID): List<Track>
}
