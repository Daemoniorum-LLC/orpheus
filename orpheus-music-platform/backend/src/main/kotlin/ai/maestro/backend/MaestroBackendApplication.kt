package ai.maestro.backend

import org.springframework.boot.autoconfigure.SpringBootApplication
import org.springframework.boot.runApplication

@SpringBootApplication
class MaestroBackendApplication

fun main(args: Array<String>) {
    runApplication<MaestroBackendApplication>(*args)
}
