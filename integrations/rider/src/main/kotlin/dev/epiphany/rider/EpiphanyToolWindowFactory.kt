package dev.epiphany.rider

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import java.awt.BorderLayout
import javax.swing.JButton
import javax.swing.JPanel
import javax.swing.JScrollPane
import javax.swing.JTextArea

class EpiphanyToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val output = JTextArea()
        output.isEditable = false
        output.lineWrap = true
        output.wrapStyleWord = true
        output.text = "Epiphany Rider bridge is ready. Click Refresh to capture a source-context status receipt."

        val refresh = JButton("Refresh")
        refresh.addActionListener {
            output.text = EpiphanyBridgeClient(project).status()
        }

        val panel = JPanel(BorderLayout(8, 8))
        panel.add(refresh, BorderLayout.NORTH)
        panel.add(JScrollPane(output), BorderLayout.CENTER)

        val content = ContentFactory.getInstance().createContent(panel, "", false)
        toolWindow.contentManager.addContent(content)
    }
}
