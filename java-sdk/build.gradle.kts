plugins {
    base
    alias(libs.plugins.kotlin.jvm)
}

tasks.register<Exec>("buildRust") {
    workingDir("${projectDir}/ext/eppo_client")
    commandLine("cargo", "build", "--release")
}

tasks.named("build") {
    dependsOn("buildRust")
}

// Add Gradle wrapper task
tasks.wrapper {
    gradleVersion = "8.5"
}