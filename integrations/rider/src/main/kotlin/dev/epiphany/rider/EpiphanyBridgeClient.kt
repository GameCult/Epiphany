package dev.epiphany.rider

import com.intellij.openapi.project.Project
import java.io.File
import java.nio.charset.StandardCharsets
import java.util.concurrent.TimeUnit

class EpiphanyBridgeClient(private val project: Project) {
    private val projectRoot: File = File(project.basePath ?: ".").absoluteFile
    private val epiphanyRoot: File = findEpiphanyRoot()

    fun status(): String {
        return runBridge(listOf("status", "--project-root", projectRoot.absolutePath))
    }

    fun captureContext(filePath: String, startLine: Int?, endLine: Int?): String {
        val args = mutableListOf(
            "context",
            "--project-root",
            projectRoot.absolutePath,
            "--file",
            filePath,
        )
        if (startLine != null) {
            args.add("--selection-start")
            args.add(startLine.toString())
        }
        if (endLine != null) {
            args.add("--selection-end")
            args.add(endLine.toString())
        }
        return runBridge(args)
    }

    private fun runBridge(args: List<String>): String {
        val script = File(epiphanyRoot, "tools/epiphany_rider_bridge.py")
        val artifactRoot = System.getenv("EPIPHANY_RIDER_ARTIFACT_ROOT")
            ?: File(epiphanyRoot, ".epiphany-gui/rider").absolutePath
        val command = mutableListOf(pythonPath(), script.absolutePath)
        command.addAll(args)
        command.add("--artifact-root")
        command.add(artifactRoot)

        val process = ProcessBuilder(command)
            .directory(epiphanyRoot)
            .redirectErrorStream(true)
            .start()
        val output = process.inputStream.readBytes().toString(StandardCharsets.UTF_8)
        process.waitFor(30, TimeUnit.SECONDS)
        return output.ifBlank { "Epiphany bridge returned no output." }
    }

    private fun pythonPath(): String {
        return System.getenv("EPIPHANY_PYTHON")
            ?: "C:\\Users\\Meta\\.cache\\codex-runtimes\\codex-primary-runtime\\dependencies\\python\\python.exe"
    }

    private fun findEpiphanyRoot(): File {
        System.getenv("EPIPHANY_REPO_ROOT")?.let { return File(it).absoluteFile }
        var cursor: File? = projectRoot
        while (cursor != null) {
            if (File(cursor, "tools/epiphany_rider_bridge.py").exists()) {
                return cursor
            }
            cursor = cursor.parentFile
        }
        return File("E:\\Projects\\EpiphanyAgent")
    }
}
