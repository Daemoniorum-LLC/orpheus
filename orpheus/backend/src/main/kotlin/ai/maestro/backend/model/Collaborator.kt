package ai.maestro.backend.model

import jakarta.persistence.*
import java.time.LocalDateTime
import java.util.UUID

enum class CollaboratorRole {
    OWNER,
    EDITOR,
    VIEWER
}

@Entity
@Table(
    name = "collaborators",
    uniqueConstraints = [UniqueConstraint(columnNames = ["project_id", "user_id"])]
)
data class Collaborator(
    @Id
    @GeneratedValue
    val id: UUID = UUID.randomUUID(),

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "project_id", nullable = false)
    val project: Project,

    @ManyToOne(fetch = FetchType.LAZY)
    @JoinColumn(name = "user_id", nullable = false)
    val user: User,

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 50)
    var role: CollaboratorRole,

    @Column(name = "created_at", nullable = false, updatable = false)
    val createdAt: LocalDateTime = LocalDateTime.now()
)
