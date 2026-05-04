package dev.epiphany.rider

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.ui.Messages

class EpiphanyCaptureContextAction : AnAction() {
    override fun actionPerformed(event: AnActionEvent) {
        val project = event.project ?: return
        val editor = event.getData(CommonDataKeys.EDITOR)
        val file = event.getData(CommonDataKeys.VIRTUAL_FILE)
        if (file == null) {
            Messages.showWarningDialog(project, "No file is selected.", "Epiphany")
            return
        }

        val document = editor?.document
        val selection = editor?.selectionModel
        val startLine = if (document != null && selection?.hasSelection() == true) {
            document.getLineNumber(selection.selectionStart) + 1
        } else {
            null
        }
        val endLine = if (document != null && selection?.hasSelection() == true) {
            document.getLineNumber(selection.selectionEnd) + 1
        } else {
            null
        }

        val output = EpiphanyBridgeClient(project).captureContext(file.path, startLine, endLine)
        Messages.showInfoMessage(project, output.take(1200), "Epiphany Context Captured")
    }
}
