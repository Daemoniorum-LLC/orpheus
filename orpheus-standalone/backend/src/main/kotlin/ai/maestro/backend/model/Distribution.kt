package ai.maestro.backend.model

import jakarta.persistence.*
import org.hibernate.annotations.JdbcTypeCode
import org.hibernate.type.SqlTypes
import java.time.LocalDateTime
import java.util.UUID

enum class DistributionStatus {
    DRAFT,
    SUBMITTED,
    PUBLISHED,
    FAILED
}

@Entity
@Table(name = "distributions")
data class Distribution(
    @Id
    @GeneratedValue
    val id: UUID = UUID.randomUUID(),

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "project_id", nullable = false)
    val project: Project,

    @Column(nullable = false)
    var title: String,

    @Column(nullable = false)
    var artist: String,

    @Column
    var album: String? = null,

    @Column(length = 100)
    var genre: String? = null,

    @Column(length = 12)
    var isrc: String? = null,

    @Column(length = 13)
    var upc: String? = null,

    @Column(name = "artwork_url", length = 500)
    var artworkUrl: String? = null,

    @JdbcTypeCode(SqlTypes.JSON)
    @Column(columnDefinition = "jsonb")
    var platforms: List<String>? = null,

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 50)
    var status: DistributionStatus = DistributionStatus.DRAFT,

    @Column(name = "submitted_at")
    var submittedAt: LocalDateTime? = null,

    @Column(name = "published_at")
    var publishedAt: LocalDateTime? = null,

    @Column(name = "created_at", nullable = false, updatable = false)
    val createdAt: LocalDateTime = LocalDateTime.now(),

    @Column(name = "updated_at", nullable = false)
    var updatedAt: LocalDateTime = LocalDateTime.now()
)
