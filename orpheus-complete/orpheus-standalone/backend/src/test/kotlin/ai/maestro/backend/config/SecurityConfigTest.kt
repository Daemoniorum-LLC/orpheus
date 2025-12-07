package ai.maestro.backend.config

import jakarta.servlet.FilterChain
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import org.junit.jupiter.api.Assertions.*
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.mockito.kotlin.*
import java.util.UUID

class DevAuthFilterTest {

    private lateinit var filter: DevAuthFilter
    private lateinit var request: HttpServletRequest
    private lateinit var response: HttpServletResponse
    private lateinit var filterChain: FilterChain

    @BeforeEach
    fun setup() {
        filter = DevAuthFilter()
        request = mock()
        response = mock()
        filterChain = mock()
    }

    @Test
    fun `should auto-authenticate regular requests with test user ID`() {
        whenever(request.requestURI).thenReturn("/api/v1/projects")

        filter.doFilterInternal(request, response, filterChain)

        verify(request).setAttribute(eq("userId"), any<UUID>())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should skip authentication for auth endpoints`() {
        whenever(request.requestURI).thenReturn("/api/v1/auth/login")

        filter.doFilterInternal(request, response, filterChain)

        verify(request, never()).setAttribute(eq("userId"), any())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should skip authentication for api-docs endpoints`() {
        whenever(request.requestURI).thenReturn("/api-docs/swagger-config")

        filter.doFilterInternal(request, response, filterChain)

        verify(request, never()).setAttribute(eq("userId"), any())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should skip authentication for swagger-ui endpoints`() {
        whenever(request.requestURI).thenReturn("/swagger-ui/index.html")

        filter.doFilterInternal(request, response, filterChain)

        verify(request, never()).setAttribute(eq("userId"), any())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should skip authentication for actuator endpoints`() {
        whenever(request.requestURI).thenReturn("/actuator/health")

        filter.doFilterInternal(request, response, filterChain)

        verify(request, never()).setAttribute(eq("userId"), any())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should authenticate project endpoints`() {
        whenever(request.requestURI).thenReturn("/api/v1/projects/123")

        filter.doFilterInternal(request, response, filterChain)

        verify(request).setAttribute(eq("userId"), any<UUID>())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should authenticate track endpoints`() {
        whenever(request.requestURI).thenReturn("/api/v1/tracks/456")

        filter.doFilterInternal(request, response, filterChain)

        verify(request).setAttribute(eq("userId"), any<UUID>())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should use consistent test user ID`() {
        whenever(request.requestURI).thenReturn("/api/v1/projects")

        var capturedUserId: UUID? = null
        whenever(request.setAttribute(eq("userId"), any<UUID>())).then { invocation ->
            capturedUserId = invocation.getArgument(1)
        }

        filter.doFilterInternal(request, response, filterChain)

        assertNotNull(capturedUserId)
        assertEquals("00000000-0000-0000-0000-000000000001", capturedUserId.toString())
    }

    @Test
    fun `should always call filter chain`() {
        whenever(request.requestURI).thenReturn("/api/v1/projects")

        filter.doFilterInternal(request, response, filterChain)

        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should handle root path`() {
        whenever(request.requestURI).thenReturn("/")

        filter.doFilterInternal(request, response, filterChain)

        verify(request).setAttribute(eq("userId"), any<UUID>())
        verify(filterChain).doFilter(request, response)
    }

    @Test
    fun `should handle nested auth paths`() {
        whenever(request.requestURI).thenReturn("/api/v1/auth/register")

        filter.doFilterInternal(request, response, filterChain)

        verify(request, never()).setAttribute(eq("userId"), any())
        verify(filterChain).doFilter(request, response)
    }
}

class SecurityConfigPasswordEncoderTest {

    @Test
    fun `should provide BCrypt password encoder`() {
        val config = SecurityConfig()
        val encoder = config.passwordEncoder()

        assertNotNull(encoder)
        assertTrue(encoder.javaClass.simpleName.contains("BCrypt"))
    }

    @Test
    fun `should encode passwords`() {
        val config = SecurityConfig()
        val encoder = config.passwordEncoder()

        val rawPassword = "password123"
        val encoded = encoder.encode(rawPassword)

        assertNotNull(encoded)
        assertNotEquals(rawPassword, encoded)
        assertTrue(encoded.length > rawPassword.length)
    }

    @Test
    fun `should match encoded passwords`() {
        val config = SecurityConfig()
        val encoder = config.passwordEncoder()

        val rawPassword = "password123"
        val encoded = encoder.encode(rawPassword)

        assertTrue(encoder.matches(rawPassword, encoded))
        assertFalse(encoder.matches("wrongpassword", encoded))
    }

    @Test
    fun `should produce different hashes for same password`() {
        val config = SecurityConfig()
        val encoder = config.passwordEncoder()

        val password = "password123"
        val hash1 = encoder.encode(password)
        val hash2 = encoder.encode(password)

        assertNotEquals(hash1, hash2)
        assertTrue(encoder.matches(password, hash1))
        assertTrue(encoder.matches(password, hash2))
    }
}
