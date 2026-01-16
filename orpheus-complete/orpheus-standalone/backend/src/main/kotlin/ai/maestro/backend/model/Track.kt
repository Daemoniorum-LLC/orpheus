package ai.maestro.backend.model

import jakarta.persistence.*
import org.hibernate.annotations.JdbcTypeCode
import org.hibernate.type.SqlTypes
import java.time.LocalDateTime
import java.util.UUID

@Entity
@Table(name = "tracks")
data class Track(
    @Id
    @GeneratedValue
    val id: UUID = UUID.randomUUID(),

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "project_id", nullable = false)
    val project: Project,

    @Column(nullable = false)
    var name: String,

    @Column(nullable = false, length = 100)
    var instrument: String,

    @Column(name = "track_number", nullable = false)
    var trackNumber: Int,

    @JdbcTypeCode(SqlTypes.JSON)
    @Column(columnDefinition = "jsonb")
    var tablature: Map<String, Any>? = null,

    @Column(name = "audio_file_url", length = 500)
    var audioFileUrl: String? = null,

    @JdbcTypeCode(SqlTypes.JSON)
    @Column(columnDefinition = "jsonb")
    var processors: Map<String, Any>? = null,

    @Column(nullable = false)
    var volume: Float = 0.75f,

    @Column(nullable = false)
    var pan: Float = 0.5f,

    @Column(nullable = false)
    var muted: Boolean = false,

    @Column(nullable = false)
    var soloed: Boolean = false,

    @Column(name = "created_at", nullable = false, updatable = false)
    val createdAt: LocalDateTime = LocalDateTime.now(),

    @Column(name = "updated_at", nullable = false)
    var updatedAt: LocalDateTime = LocalDateTime.now()
)
