package ai.maestro.backend.config

import jakarta.servlet.FilterChain
import jakarta.servlet.http.HttpServletRequest
import jakarta.servlet.http.HttpServletResponse
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty
import org.springframework.context.annotation.Bean
import org.springframework.context.annotation.Configuration
import org.springframework.security.config.annotation.web.builders.HttpSecurity
import org.springframework.security.config.annotation.web.configuration.EnableWebSecurity
import org.springframework.security.config.http.SessionCreationPolicy
import org.springframework.security.crypto.bcrypt.BCryptPasswordEncoder
import org.springframework.security.crypto.password.PasswordEncoder
import org.springframework.security.web.SecurityFilterChain
import org.springframework.security.web.authentication.UsernamePasswordAuthenticationFilter
import org.springframework.stereotype.Component
import org.springframework.web.filter.OncePerRequestFilter
import java.util.UUID

@Configuration
@EnableWebSecurity
class SecurityConfig {

    @Bean
    fun securityFilterChain(
        http: HttpSecurity,
        devAuthFilter: DevAuthFilter
    ): SecurityFilterChain {
        http
            .csrf { it.disable() }
            .cors { }
            .sessionManagement { it.sessionCreationPolicy(SessionCreationPolicy.STATELESS) }
            .authorizeHttpRequests { auth ->
                auth
                    .requestMatchers(
                        "/api/v1/auth/**",
                        "/api-docs/**",
                        "/swagger-ui/**",
                        "/actuator/**"
                    ).permitAll()
                    .anyRequest().authenticated()
            }
            .addFilterBefore(devAuthFilter, UsernamePasswordAuthenticationFilter::class.java)

        return http.build()
    }

    @Bean
    fun passwordEncoder(): PasswordEncoder = BCryptPasswordEncoder()
}

/**
 * Development-only authentication filter
 * Automatically authenticates all requests with a test user ID
 *
 * SECURITY WARNING: This filter bypasses all authentication!
 * Only enable in local development environments.
 *
 * To enable: Set app.dev-auth.enabled=true in application.yaml
 * Production deployments MUST NOT enable this property.
 */
@Component
@ConditionalOnProperty(
    name = ["app.dev-auth.enabled"],
    havingValue = "true",
    matchIfMissing = false  // Disabled by default for security
)
class DevAuthFilter : OncePerRequestFilter() {

    private val testUserId = UUID.fromString("00000000-0000-0000-0000-000000000001")

    override fun doFilterInternal(
        request: HttpServletRequest,
        response: HttpServletResponse,
        filterChain: FilterChain
    ) {
        // Skip auth endpoints
        if (request.requestURI.startsWith("/api/v1/auth") ||
            request.requestURI.startsWith("/api-docs") ||
            request.requestURI.startsWith("/swagger-ui") ||
            request.requestURI.startsWith("/actuator")
        ) {
            filterChain.doFilter(request, response)
            return
        }

        // For development: auto-authenticate with test user
        request.setAttribute("userId", testUserId)
        filterChain.doFilter(request, response)
    }
}
