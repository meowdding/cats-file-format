plugins {
    id("maven-publish")
    id("java")
}

group = "me.owdding"
version = "1.0.0-beta.1"

java {
    withSourcesJar()
}

tasks {
    compileJava {
        options.encoding = "UTF-8"
    }
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
        }
    }

    repositories {
        maven {
            setUrl("https://maven.teamresourceful.com/repository/thatgravyboat/")

            credentials {
                username = System.getenv("MAVEN_USER") ?: providers.gradleProperty("maven_username").orNull
                password = System.getenv("MAVEN_PASS") ?: providers.gradleProperty("maven_password").orNull
            }
        }
    }
}