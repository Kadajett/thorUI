import java.io.File

plugins {
    id("com.android.application")
}

val repositoryRoot = rootProject.layout.projectDirectory.dir("../..").asFile
val webDistribution = File(repositoryRoot, "dist")
val releaseVersion = providers.gradleProperty("thoruiVersion").orElse("0.1.0-alpha.1")
val signingStore = providers.environmentVariable("THORUI_SIGNING_STORE_FILE")

android {
    namespace = "dev.yougotserved.thorui.demo"
    compileSdk = 37

    defaultConfig {
        applicationId = "dev.yougotserved.thorui.demo"
        minSdk = 26
        targetSdk = 37
        versionCode = 1
        versionName = releaseVersion.get()
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    signingConfigs {
        create("release") {
            if (signingStore.isPresent) {
                storeFile = file(signingStore.get())
                storePassword = providers.environmentVariable("THORUI_SIGNING_STORE_PASSWORD").get()
                keyAlias = providers.environmentVariable("THORUI_SIGNING_KEY_ALIAS").get()
                keyPassword = providers.environmentVariable("THORUI_SIGNING_KEY_PASSWORD").get()
            }
        }
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
            if (signingStore.isPresent) signingConfig = signingConfigs.getByName("release")
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    buildFeatures {
        buildConfig = true
    }

    sourceSets.getByName("main").assets.directories.add(webDistribution.absolutePath)
}

dependencies {
    implementation("androidx.webkit:webkit:1.17.0")
    testImplementation("junit:junit:4.13.2")
}
