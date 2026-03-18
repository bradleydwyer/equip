# Xcode MCP Bridge (`xcrun mcpbridge`) Research

Status: Current as of March 2026 (Xcode 26.3)

## Overview

Apple ships a built-in MCP server with Xcode 26.3+. The binary `xcrun mcpbridge` translates MCP protocol requests into Xcode's internal XPC calls, allowing external AI coding tools (Claude Code, Codex, Cursor, Gemini CLI, etc.) to interact with Xcode projects.

## Setup

### Enable in Xcode
Settings (Cmd-,) > Intelligence > Model Context Protocol > Xcode Tools > ON

### Claude Code
```bash
claude mcp add --transport stdio xcode -- xcrun mcpbridge
```

### Codex
```bash
codex mcp add xcode -- xcrun mcpbridge
```

### Cursor
Add to `~/.cursor/mcp.json`:
```json
{
  "mcpServers": {
    "xcode-tools": {
      "command": "xcrun",
      "args": ["mcpbridge"]
    }
  }
}
```

## Complete Tool List (24 tools, Xcode 26.3 RC 1)

Source: [GitHub Gist - keith/d8aca9661002388650cf2fdc5eac9f3b](https://gist.github.com/keith/d8aca9661002388650cf2fdc5eac9f3b)

### Discovery

| Tool | Description | Parameters |
|------|-------------|------------|
| **XcodeListWindows** | Lists current Xcode windows and workspace info | *(none required)* |

Most other tools require `tabIdentifier` (string) -- obtained from `XcodeListWindows`.

### File Operations

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| **XcodeRead** | Read file contents | `tabIdentifier`, `filePath`, `limit?`, `offset?` |
| **XcodeWrite** | Create or overwrite files | `tabIdentifier`, `filePath`, `content` |
| **XcodeUpdate** | Edit files via string replacement | `tabIdentifier`, `filePath`, `oldString`, `newString`, `replaceAll?` |
| **XcodeLS** | List directory contents | `tabIdentifier`, `path`, `recursive?`, `ignore?` (string array) |
| **XcodeGlob** | Find files by wildcard pattern | `tabIdentifier`, `pattern?`, `path?` |
| **XcodeGrep** | Regex search across files | `tabIdentifier`, `pattern`, `path?`, `glob?`, `ignoreCase?`, `multiline?`, `linesContext?`, `linesBefore?`, `linesAfter?`, `showLineNumbers?`, `outputMode?` (content/filesWithMatches/count), `headLimit?`, `type?` |
| **XcodeMakeDir** | Create directories/groups | `tabIdentifier`, `directoryPath` |
| **XcodeRM** | Remove files/directories | `tabIdentifier`, `path`, `deleteFiles?`, `recursive?` |
| **XcodeMV** | Move/rename/copy files | `tabIdentifier`, `sourcePath`, `destinationPath`, `operation?` (move/copy), `overwriteExisting?` |

### Build & Test

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| **BuildProject** | Build and wait for completion | `tabIdentifier` |
| **GetBuildLog** | Get build output/diagnostics | `tabIdentifier`, `severity?` (error/warning/remark), `pattern?`, `glob?` |
| **RunAllTests** | Run all tests from active test plan | `tabIdentifier` |
| **RunSomeTests** | Run specific tests | `tabIdentifier`, `tests` (array of `{targetName, testIdentifier}`) |
| **GetTestList** | List available tests | `tabIdentifier` |

### Diagnostics

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| **XcodeListNavigatorIssues** | List issues from Issue Navigator | `tabIdentifier`, `severity?`, `pattern?`, `glob?` |
| **XcodeRefreshCodeIssuesInFile** | Get live compiler diagnostics for a file | `tabIdentifier`, `filePath` |

### Execution & Preview

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| **ExecuteSnippet** | Run Swift code in REPL context | `tabIdentifier`, `codeSnippet`, `sourceFilePath`, `timeout?` |
| **RenderPreview** | Render SwiftUI preview as image | `tabIdentifier`, `sourceFilePath`, `previewDefinitionIndexInFile?`, `timeout?` |

### Documentation

| Tool | Description | Key Parameters |
|------|-------------|----------------|
| **DocumentationSearch** | Semantic search of Apple docs + WWDC transcripts | `query`, `frameworks?` (string array) |

Note: DocumentationSearch does NOT require `tabIdentifier`. It uses "Squirrel MLX" -- Apple's MLX-accelerated embedding system on Apple Silicon.

The gist shows 24 tools total (3 additional beyond the 20 commonly cited -- likely minor variants or internal tools).

## Typical Agent Workflow

1. Agent calls `XcodeListWindows()` to discover open projects and get `tabIdentifier`
2. Agent reads/explores project with `XcodeLS`, `XcodeRead`, `XcodeGlob`, `XcodeGrep`
3. Agent makes changes with `XcodeWrite` or `XcodeUpdate`
4. Agent calls `BuildProject` to compile (incremental builds ~0.9s)
5. Agent checks `GetBuildLog` for errors or `XcodeRefreshCodeIssuesInFile` for diagnostics
6. Agent iterates on fixes until build succeeds
7. Agent runs tests with `RunSomeTests` or `RunAllTests`
8. Agent can render SwiftUI previews with `RenderPreview` for visual verification
9. Agent can search Apple docs with `DocumentationSearch` for API guidance

## Known Issues & Limitations

### Single Connection / Security Prompts
- stdio transport supports only one connection at a time
- Each new `mcpbridge` process triggers a macOS security dialog
- Workaround: use `mcp-proxy` to maintain a persistent connection and avoid repeated prompts
- See: [Fix Xcode MCP Spam Security Prompts](https://scottwhill.com/thoughts/fix-xcode-mcp-spam-security-prompts-in-codex-claude-code-mcp-proxy)

### Requires Running Xcode
- Xcode must be running with a project open
- Does not work for CI/CD or headless workflows
- mcpbridge auto-detects Xcode PID; uses `xcode-select` when multiple instances exist

### Sandboxed Environment
- Xcode creates a restricted shell that does NOT inherit `.zshrc`, `.bashrc`, or `PATH`
- MCP configs must use absolute paths and explicitly define environment variables

### Configuration Locations (for Xcode-embedded agent)
- Claude: `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/.claude`
- Codex: `~/Library/Developer/Xcode/CodingAssistant/codex`
- Skills: `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/skills`
- Commands: `~/Library/Developer/Xcode/CodingAssistant/ClaudeAgentConfig/commands`
- Global `~/.claude.json` is ignored when running inside Xcode

### SPM Multi-Target Projects
- Can cause confusion about build schemes
- Recommended: add scheme info to `CLAUDE.md` / `AGENTS.md`

### structuredContent (Fixed in RC 2)
- RC 1 returned data in `content` but not `structuredContent` (MCP spec violation)
- Fixed in Xcode 26.3 RC 2

## Complementary MCP Servers

The native Xcode MCP bridge does NOT cover simulators, debugging, or UI automation. For those, consider:

- **XcodeBuildMCP** (Sentry): Adds `simulator/build`, `simulator/build-and-run`, `simulator/test`, `simulator/screenshot`, `debugging/attach`, `debugging/breakpoint`, `ui-automation/tap`, `ui-automation/swipe` -- [github.com/getsentry/XcodeBuildMCP](https://github.com/getsentry/XcodeBuildMCP)

## Sources

- [Apple Developer Documentation: Giving external agentic coding tools access to Xcode](https://developer.apple.com/documentation/xcode/giving-agentic-coding-tools-access-to-xcode)
- [Apple Tech Talk: Meet agentic coding in Xcode](https://developer.apple.com/videos/play/tech-talks/111428/)
- [Xcode 26.3 RC 1 MCP tools/list response (GitHub Gist)](https://gist.github.com/keith/d8aca9661002388650cf2fdc5eac9f3b)
- [Rudrank Riyam: Exploring Xcode MCP Tools in Cursor, Claude Code and Codex](https://rudrank.com/exploring-xcode-using-mcp-tools-cursor-external-clients)
- [BleepingSwift: How to Use Xcode's MCP Server](https://bleepingswift.com/blog/xcode-mcp-server-ai-workflow)
- [Fatbobman: Xcode 26.3 + Claude Agent](https://fatbobman.com/en/posts/xcode-263-claude/)
- [Marc0.dev: Xcode 26.3 Claude Agent - What Works and What Doesn't](https://www.marc0.dev/en/blog/ai-agents/xcode-26-3-claude-agent-what-actually-works-and-what-doesnt-1770494531265)
- [Scott W Hill: Fix Xcode MCP Spam Security Prompts](https://scottwhill.com/thoughts/fix-xcode-mcp-spam-security-prompts-in-codex-claude-code-mcp-proxy)
- [DEV Community: Xcode 26.3 - Use AI Agents from Cursor, Claude Code & Beyond](https://dev.to/arshtechpro/xcode-263-use-ai-agents-from-cursor-claude-code-beyond-4dmi)
