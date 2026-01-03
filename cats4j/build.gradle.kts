plugins {
    id("java")
}

group = "me.owdding"
version = "1.0.0"

java {
    withSourcesJar()
}

tasks {
    compileJava {
        options.encoding = "UTF-8"
    }
}