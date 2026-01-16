package ai.maestro.backend.repository

import ai.maestro.backend.model.Project
import ai.maestro.backend.model.User
import org.springframework.data.jpa.repository.JpaRepository
import org.springframework.data.jpa.repository.Query
import org.springframework.stereotype.Repository
import java.util.UUID

@Repository
interface ProjectRepository : JpaRepository<Project, UUID> {
    fun findByUser(user: User): List<Project>
    fun findByUserOrderByCreatedAtDesc(user: User): List<Project>

    @Query("""
        SELECT DISTINCT p FROM Project p
        LEFT JOIN p.collaborators c
        WHERE p.user.id = :userId OR c.user.id = :userId
        ORDER BY p.createdAt DESC
    """)
    fun findAllAccessibleByUserId(userId: UUID): List<Project>
}
